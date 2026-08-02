use core::f64;
use std::sync::Arc;

use crate::measures::curves::{
    approxdp::{beta_via_approxDP, delta_via_approxDP, epsilon_via_approxdp},
    profile::{beta_via_profile, delta_via_profile},
    tradeoff::delta_via_tradeoff,
};
use dashu::rational::RBig;

use crate::{
    error::{ErrorVariant, Fallible},
    measures::privacy_profile::{delta_to_log_lower, delta_to_log_upper},
    traits::DInterval,
    utilities::search::{Above, fallible_binary_search},
};

mod approxdp;
mod profile;
mod tradeoff;

#[cfg(test)]
mod test;

#[deprecated(since = "0.15.0", note = "Use PrivacyCurve instead.")]
#[allow(dead_code)] // compatibility alias is consumed by later stack commits
/// Compatibility alias while callers migrate to [`PrivacyCurve`].
pub type PrivacyProfile = PrivacyCurve;

/// A unified representation of privacy guarantees that can be queried as either
/// a privacy profile `delta(epsilon)` or an f-DP tradeoff curve `beta(alpha)`.
#[derive(Clone, Default)]
pub struct PrivacyCurve {
    delta_slack: f64,
    // invariant: order increasing in epsilon, nonincreasing in delta
    approx_dp: Option<Arc<[ApproxDPPoint]>>,
    profile: Option<Profile>,
    tradeoff: Option<Tradeoff>,
}

#[derive(Clone)]
struct Profile {
    log_delta: Arc<ProfileFn>,
    epsilon: Option<Arc<EpsilonFn>>,
}

type DeltaFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

impl Profile {
    fn new(log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        Self {
            log_delta: Arc::new(move |epsilon| checked_log_delta_output(log_delta(epsilon)?)),
            epsilon: None,
        }
    }

    fn new_with_epsilon(
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        epsilon: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Self {
        Self {
            log_delta: Arc::new(move |value| checked_log_delta_output(log_delta(value)?)),
            epsilon: Some(Arc::new(epsilon)),
        }
    }

    fn new_from_delta(delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        let delta: Arc<DeltaFn> = Arc::new(delta);
        let forward = delta.clone();
        let reverse = delta.clone();

        Self::new_with_epsilon(
            move |epsilon| delta_to_log_upper(checked_delta_output(forward(epsilon)?)?),
            move |target_delta| invert_delta_callback(reverse.as_ref(), target_delta),
        )
    }
}

#[derive(Clone)]
struct Tradeoff {
    beta: Arc<TradeoffFn>,
    symmetric: bool,
}

type ProfileFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type EpsilonFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type TradeoffFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

#[derive(Clone, Debug)]
pub(crate) struct ApproxDPPoint {
    epsilon: f64,
    delta: f64,
    // allows to cache computations that are repeated many times in tradeoff evaluation
    one_minus_delta: RBig,
    exp_eps_up: RBig,
    exp_neg_eps_down: RBig,
}

impl PrivacyCurve {
    pub fn new() -> Self {
        Default::default()
    }

    /// Construct an (ε, δ)-DP privacy profile from epsilon-delta pairs.
    ///
    /// # Arguments
    /// * `pairs` - a vector of approx-DP pairs
    pub fn with_approxDP(mut self, mut points: Vec<(f64, f64)>) -> Fallible<Self> {
        if points.is_empty() {
            return fallible!(
                FailedMap,
                "privacy curve must be defined by at least one approximate-DP pair"
            );
        }

        points.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| b.1.total_cmp(&a.1)));
        // For duplicate epsilons, keep the largest delta conservatively.
        points.dedup_by(|a, b| a.0 == b.0);
        // Keep the earliest epsilon for each delta plateau.
        points.dedup_by(|later, earlier| later.1 == earlier.1);

        let mut min_delta = 1.0;
        for (epsilon, delta) in &points {
            if !epsilon.is_finite() {
                return fallible!(FailedMap, "epsilon values in privacy curve must be finite");
            }
            if *delta > min_delta {
                return fallible!(
                    FailedMap,
                    "delta values must be monotonically nonincreasing as epsilon increases"
                );
            }
            min_delta = min_delta.min(*delta);
        }

        self.approx_dp = Some(Arc::from(
            points
                .into_iter()
                .map(ApproxDPPoint::build)
                .collect::<Fallible<Vec<_>>>()?
                .into_boxed_slice(),
        ));

        Ok(self)
    }

    /// Attach an additive catastrophic failure probability to the privacy curve.
    ///
    /// This represents a representation-independent delta slack that is added to
    /// the privacy profile `delta(epsilon)`, allowing curves such as
    /// approximate-zCDP to be expressed as a concentrated/privacy-profile
    /// representation plus a fixed catastrophic failure parameter.
    pub fn with_delta_slack(mut self, delta_slack: f64) -> Fallible<Self> {
        check_delta(delta_slack)?;
        self.delta_slack = delta_slack;
        Ok(self)
    }

    /// Construct a privacy curve from a callback mapping `epsilon -> delta`.
    ///
    /// For tight conversion to f-DP, the profile should also preserve the
    /// hockey-stick structure of true privacy profiles:
    ///
    /// * λ ↦ δ(log λ) is convex and nonincreasing for λ >= 1
    ///
    /// If this property is not satisfied, `beta(alpha)` remains conservative,
    /// but may be loose because the optimizer may miss the best epsilon.
    ///
    /// # Arguments
    /// * `curve` - A privacy profile mapping epsilon to delta
    ///
    /// # Why honest-but-curious?
    ///
    /// The privacy profile should implement a well-defined $\delta(\epsilon)$ curve:
    ///
    /// * is functionally pure
    /// * nonincreasing
    /// * returns delta values only within $[0, 1]$
    /// * returned values are upward-conservative if numerically approximate
    #[cfg(feature = "honest-but-curious")]
    pub fn with_profile(
        mut self,
        delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.profile = Some(Profile::new_from_delta(delta));
        Ok(self)
    }

    /// Construct a privacy curve from a callback mapping `epsilon -> log(delta)`.
    ///
    /// For tight conversion to f-DP, the profile should also preserve the
    /// hockey-stick structure of true privacy profiles:
    ///
    /// * λ ↦ δ(log λ) is convex and nonincreasing for λ >= 1
    ///
    /// If this property is not satisfied, `beta(alpha)` remains conservative,
    /// but may be loose because the optimizer may miss the best epsilon.
    ///
    /// # Arguments
    /// * `curve` - A privacy profile mapping epsilon to delta
    ///
    /// # Why honest-but-curious?
    ///
    /// The privacy profile should implement a well-defined $\delta(\epsilon)$ curve:
    ///
    /// * is functionally pure
    /// * nonincreasing
    /// * returns log(delta), where delta is within $[0, 1]$
    /// * returned values are upward-conservative if numerically approximate
    #[cfg(feature = "honest-but-curious")]
    pub fn with_log_profile(
        mut self,
        delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.profile = Some(Profile::new(delta));
        Ok(self)
    }

    /// Construct a log-delta profile with an independently certified inverse.
    #[cfg(feature = "honest-but-curious")]
    pub(crate) fn with_log_profile_with_epsilon(
        mut self,
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        epsilon: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.profile = Some(Profile::new_with_epsilon(log_delta, epsilon));
        Ok(self)
    }

    /// Construct a symmetric tradeoff function from a callback mapping `alpha -> beta`.
    ///
    /// # Arguments
    /// * `curve` - An $f$-DP tradeoff curve mapping alpha to beta
    ///
    /// # Why honest-but-curious?
    ///
    /// The tradeoff curve should implement a well-defined $\beta(\alpha)$ curve.
    ///
    /// * is functionally pure
    /// * returns finite beta values in [0, 1]
    /// * satisfies β(0) = 1 and β(1) = 0
    /// * is nonincreasing and convex on [0, 1]
    /// * returns downward-conservative beta values if numerically approximate
    /// * beta(beta(alpha)) = alpha
    #[cfg(feature = "honest-but-curious")]
    pub fn with_symmetric_tradeoff(
        mut self,
        beta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.tradeoff = Some(Tradeoff {
            beta: Arc::new(beta),
            symmetric: true,
        });
        Ok(self)
    }

    /// Construct a tradeoff function from a callback mapping `alpha -> beta`.
    ///
    /// # Arguments
    /// * `curve` - An $f$-DP tradeoff curve mapping alpha to beta
    ///
    /// # Why honest-but-curious?
    ///
    /// The tradeoff curve should implement a well-defined $\beta(\alpha)$ curve:
    ///
    /// * is functionally pure
    /// * returns finite beta values in [0, 1]
    /// * satisfies beta(0) = 1 and beta(1) = 0
    /// * is nonincreasing and convex on [0, 1]
    /// * returns downward-conservative beta values if numerically approximate
    #[cfg(feature = "honest-but-curious")]
    pub fn with_tradeoff(
        mut self,
        beta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.tradeoff = Some(Tradeoff {
            beta: Arc::new(beta),
            symmetric: false,
        });
        Ok(self)
    }

    /// Evaluate the privacy profile at `epsilon`.
    ///
    /// # Arguments
    /// * `epsilon` - What to fix epsilon to compute delta.
    fn delta_base(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;

        if epsilon.is_infinite() {
            return Ok(0.0);
        }

        let delta = if let Some(Profile { log_delta, .. }) = &self.profile {
            delta_via_profile(log_delta.as_ref(), epsilon)
        } else if let Some(points) = &self.approx_dp {
            delta_via_approxDP(points, epsilon)
        } else if let Some(Tradeoff { beta, symmetric }) = &self.tradeoff {
            delta_via_tradeoff(beta.as_ref(), *symmetric, epsilon)
        } else {
            return fallible!(FailedFunction, "PrivacyCurve has no representation");
        }?;

        check_delta(delta)?;
        Ok(delta)
    }

    /// Return a conservative upper bound on delta at the given nonnegative epsilon.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        let mut delta = self.delta_base(epsilon)?;

        if self.delta_slack > 0.0 {
            delta = DInterval::point(delta)?
                .add(DInterval::point(self.delta_slack)?)?
                .upper_f64()?
                .clamp(0.0, 1.0);
        }

        check_delta(delta)?;
        Ok(delta)
    }

    /// Evaluate the f-DP tradeoff curve at `alpha`.
    ///
    /// # Arguments
    /// * `alpha` - What to fix alpha to compute beta.
    fn beta_base(&self, alpha: f64) -> Fallible<f64> {
        check_alpha(alpha)?;

        if alpha == 0.0 {
            return Ok(1.0);
        }
        if alpha == 1.0 {
            return Ok(0.0);
        }

        let beta = if let Some(Tradeoff { beta, .. }) = &self.tradeoff {
            beta(alpha)?
        } else if let Some(Profile { log_delta, .. }) = &self.profile {
            beta_via_profile(log_delta.as_ref(), alpha)?
        } else if let Some(points) = &self.approx_dp {
            beta_via_approxDP(points, alpha)?
        } else {
            return fallible!(FailedFunction, "PrivacyCurve has no representation");
        };

        check_beta(beta)?;
        Ok(beta)
    }

    /// Return a conservative lower bound on beta at the given alpha in `[0, 1]`.
    pub fn beta(&self, alpha: f64) -> Fallible<f64> {
        if self.delta_slack == 0.0 {
            return self.beta_base(alpha);
        }
        let curve = self.clone();
        // TODO: this could be pushed deeper into the calculation for efficiency
        beta_via_profile(
            &move |epsilon| delta_to_log_upper(curve.delta(epsilon)?),
            alpha,
        )
    }

    /// Invert the privacy curve by finding the smallest `epsilon`
    /// such that `delta(epsilon) <= delta`.
    ///
    /// # Arguments
    /// * `delta` - What to fix delta to compute epsilon.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;

        if delta == 1.0 {
            return Ok(0.0);
        }
        if delta == 0.0 && self.profile.is_some() && self.approx_dp.is_none() {
            return Ok(if self.delta_base(0.0)? == 0.0 {
                0.0
            } else {
                f64::INFINITY
            });
        }

        if delta < self.delta_slack {
            return Ok(f64::INFINITY);
        }
        let remaining_delta = DInterval::point(delta)?
            .sub(DInterval::point(self.delta_slack)?)?
            .lower_f64()?
            .clamp(0.0, 1.0);

        if self.delta_slack == 0.0 {
            if let Some(Profile { log_delta, epsilon }) = &self.profile {
                if let Some(epsilon) = epsilon {
                    let value = epsilon(remaining_delta)?;
                    if value.is_nan() || value.is_sign_negative() {
                        return fallible!(
                            FailedMap,
                            "epsilon ({value}) must be non-negative and not NaN"
                        );
                    }
                    return Ok(value.max(0.0));
                }

                return invert_log_profile(log_delta.as_ref(), remaining_delta);
            }
        }

        // Fast path only when ApproxDP is the preferred delta representation.
        // If profile exists, self.delta(...) would use profile, so do not bypass it.
        if self.profile.is_none() {
            if let Some(points) = &self.approx_dp {
                return epsilon_via_approxdp(points, remaining_delta);
            }
        }

        if self.delta(0.0)? <= delta {
            return Ok(0.0);
        }
        match fallible_binary_search(|epsilon| Ok(self.delta(*epsilon)? <= delta), Above(0.0)) {
            Ok(epsilon) => Ok(epsilon),
            Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
            Err(err) => Err(err),
        }
    }

    /// Returns a conservative lower bound on the smallest alpha such that
    /// beta(alpha) <= beta.
    ///
    /// # Arguments
    /// * `beta` - What to fix beta to compute alpha.
    pub fn alpha(&self, beta: f64) -> Fallible<f64> {
        check_beta(beta)?;

        if beta == 0.0 {
            return Ok(1.0);
        }
        if beta == 1.0 {
            return Ok(0.0);
        }

        if self.delta_slack != 0.0 {
            return self.alpha_by_inverting_beta(beta);
        }

        let alpha = if let Some(Tradeoff {
            beta: beta_fn,
            symmetric,
        }) = &self.tradeoff
        {
            if *symmetric {
                // For symmetric tradeoff curves, alpha(beta) == beta(beta).
                beta_fn(beta)?
            } else {
                // Non-symmetric tradeoff is still the preferred beta representation,
                // so invert the preferred beta path instead of falling through.
                return self.alpha_by_inverting_beta(beta);
            }
        } else if let Some(Profile { log_delta, .. }) = &self.profile {
            beta_via_profile(log_delta.as_ref(), beta)?
        } else if let Some(points) = &self.approx_dp {
            beta_via_approxDP(points, beta)?
        } else {
            return fallible!(FailedFunction, "PrivacyCurve has no representation");
        };

        check_alpha(alpha)?;
        Ok(alpha)
    }

    fn alpha_by_inverting_beta(&self, beta: f64) -> Fallible<f64> {
        let passing = fallible_binary_search(|alpha| Ok(self.beta(*alpha)? <= beta), (0.0, 1.0))?;
        Ok(passing.next_down().clamp(0.0, 1.0))
    }

    #[allow(dead_code)] // composition is wired into measurements later in the stack
    pub(crate) fn compose(curves: Vec<Self>) -> Fallible<Self> {
        let delta_slack = curves.iter().try_fold(0.0, |acc, curve| {
            check_delta(curve.delta_slack)?;
            DInterval::point(acc)?
                .add(DInterval::point(curve.delta_slack)?)?
                .upper_f64()
                .map(|value| value.min(1.0))
        })?;

        let mut out = PrivacyCurve::new().with_delta_slack(delta_slack)?;
        if curves.is_empty() {
            out.approx_dp = Some(Arc::from(
                vec![ApproxDPPoint::build((0.0, 0.0))?].into_boxed_slice(),
            ));
            return Ok(out);
        }

        let mut composed_any_base_repr = false;

        if let Some(points) = compose_singleton_approxDP(&curves)? {
            out.approx_dp = Some(points);
            composed_any_base_repr = true;
        }

        if !composed_any_base_repr && curves.iter().any(PrivacyCurve::has_base_repr) {
            return fallible!(
                FailedFunction,
                "PrivacyCurve composition requires a common composition representation"
            );
        }

        if !composed_any_base_repr {
            out.approx_dp = Some(Arc::from(
                vec![ApproxDPPoint::build((0.0, 0.0))?].into_boxed_slice(),
            ));
        }

        Ok(out)
    }

    fn has_base_repr(&self) -> bool {
        self.approx_dp.is_some() || self.profile.is_some() || self.tradeoff.is_some()
    }
}

#[allow(dead_code)] // called by PrivacyCurve::compose later in the stack
fn compose_singleton_approxDP(curves: &[PrivacyCurve]) -> Fallible<Option<Arc<[ApproxDPPoint]>>> {
    let mut epsilon_sum = DInterval::point(0.0)?;
    let mut delta_sum = DInterval::point(0.0)?;
    let mut saw_non_identity = false;

    for curve in curves {
        match curve.approx_dp.as_deref() {
            Some([point]) => {
                epsilon_sum = epsilon_sum.add(DInterval::point(point.epsilon)?)?;
                delta_sum = delta_sum.add(DInterval::point(point.delta)?)?;
                saw_non_identity = true;
            }
            Some(_) => return Ok(None),
            None if !curve.has_base_repr() => {}
            None => return Ok(None),
        }
    }

    if !saw_non_identity {
        return Ok(None);
    }

    let epsilon_sum = epsilon_sum.upper_f64()?;
    if !epsilon_sum.is_finite() {
        return fallible!(Overflow, "composed epsilon is not finite");
    }
    let delta_sum = delta_sum.upper_f64()?.min(1.0);

    Ok(Some(Arc::from(
        vec![ApproxDPPoint::build((epsilon_sum, delta_sum))?].into_boxed_slice(),
    )))
}

fn invert_delta_callback(delta: &DeltaFn, target_delta: f64) -> Fallible<f64> {
    if checked_delta_output(delta(0.0)?)? <= target_delta {
        return Ok(0.0);
    }

    match fallible_binary_search(
        |epsilon| Ok(checked_delta_output(delta(*epsilon)?)? <= target_delta),
        Above(0.0),
    ) {
        Ok(epsilon) => Ok(epsilon),
        Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
        Err(err) => Err(err),
    }
}

fn invert_log_profile(profile: &ProfileFn, target_delta: f64) -> Fallible<f64> {
    if target_delta == 0.0 {
        if profile(0.0)? == f64::NEG_INFINITY {
            return Ok(0.0);
        }

        return match fallible_binary_search(
            |epsilon| Ok(profile(*epsilon)? == f64::NEG_INFINITY),
            Above(0.0),
        ) {
            Ok(epsilon) => Ok(epsilon),
            Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
            Err(err) => Err(err),
        };
    }

    let log_target_lower = delta_to_log_lower(target_delta)?;
    if profile(0.0)? <= log_target_lower {
        return Ok(0.0);
    }

    match fallible_binary_search(
        |epsilon| Ok(profile(*epsilon)? <= log_target_lower),
        Above(0.0),
    ) {
        Ok(epsilon) => Ok(epsilon),
        Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
        Err(err) => Err(err),
    }
}

fn checked_delta_output(delta: f64) -> Fallible<f64> {
    check_delta(delta)?;
    Ok(delta)
}

fn checked_log_delta_output(log_delta: f64) -> Fallible<f64> {
    if log_delta.is_nan() || log_delta > 0.0 {
        return fallible!(
            FailedMap,
            "log(delta) ({log_delta}) must be at most zero and not NaN"
        );
    }
    Ok(log_delta)
}

fn check_epsilon(epsilon: f64) -> Fallible<()> {
    if epsilon.is_nan() {
        return fallible!(FailedMap, "epsilon must not be nan");
    }
    if epsilon.is_sign_negative() {
        return fallible!(
            FailedMap,
            "epsilon ({epsilon}) must be a non-negative number"
        );
    }
    Ok(())
}
fn check_alpha(alpha: f64) -> Fallible<()> {
    check_01(alpha, "alpha")
}
fn check_beta(beta: f64) -> Fallible<()> {
    check_01(beta, "beta")
}
fn check_delta(delta: f64) -> Fallible<()> {
    check_01(delta, "delta")
}

fn check_01(value: f64, name: &str) -> Fallible<()> {
    if !value.is_finite() {
        return fallible!(FailedMap, "{name} ({value}) must be finite");
    }
    if value.is_sign_negative() || value > 1.0 {
        return fallible!(FailedMap, "{name} ({value}) must be between zero and one");
    }
    Ok(())
}
