use core::f64;
use std::sync::Arc;

use crate::measures::curves::{
    approxdp::{beta_via_approxDP, delta_via_approxDP, epsilon_via_approxdp},
    gaussiandp::{beta_via_gaussianDP, delta_via_gaussianDP},
    profile::{beta_via_profile, delta_via_profile},
    renyidp::{beta_via_renyiDP, beta_via_zCDP, delta_via_renyiDP, delta_via_zCDP},
    tradeoff::delta_via_tradeoff,
};
use dashu::rational::RBig;

use crate::error::Fallible;

#[cfg(feature = "ffi")]
mod ffi;

mod approxdp;
mod gaussiandp;
mod profile;
mod renyidp;
mod tradeoff;

#[cfg(test)]
mod test;

#[deprecated(since = "0.15.0", note = "Use PrivacyCurve instead.")]
/// Compatibility alias while callers migrate to [`PrivacyCurve`].
pub type PrivacyProfile = PrivacyCurve;

/// A unified representation of privacy guarantees that can be queried as either
/// a privacy profile `delta(epsilon)` or an f-DP tradeoff curve `beta(alpha)`.
#[derive(Clone, Default)]
pub struct PrivacyCurve {
    delta_slack: f64,
    // invariant: order increasing in epsilon, nonincreasing in delta
    approx_dp: Option<Arc<[ApproxDPPoint]>>,
    gaussian_dp: Option<f64>,
    profile: Option<Profile>,
    tradeoff: Option<Tradeoff>,
    renyi_dp: Option<Arc<RenyiFn>>,
    zcdp: Option<f64>,
}

#[derive(Clone)]
struct Profile {
    delta: Arc<ProfileFn>,
    scale: ProfileScale,
}
#[derive(Clone)]
struct Tradeoff {
    beta: Arc<TradeoffFn>,
    symmetric: bool,
}

type ProfileFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type TradeoffFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type RenyiFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProfileScale {
    Delta,
    LogDelta,
}

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

    /// Construct a privacy curve corresponding to Gaussian differential privacy with parameter `mu`.
    ///
    /// # Why idealized-numerics?
    /// While the calculations have best-effort protections against float underestimation,
    /// error bounds for transcendentals like `erfcx` are not known and could be underestimated.
    #[cfg(feature = "idealized-numerics")]
    pub fn with_gaussianDP(mut self, mu: f64) -> Fallible<Self> {
        if !mu.is_finite() || mu < 0.0 {
            return fallible!(FailedMap, "mu ({mu}) must be a finite non-negative number");
        }

        self.gaussian_dp = Some(mu);
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
        self.profile = Some(Profile {
            delta: Arc::new(delta),
            scale: ProfileScale::Delta,
        });
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
        self.profile = Some(Profile {
            delta: Arc::new(delta),
            scale: ProfileScale::LogDelta,
        });
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

    /// Construct a privacy curve from a Rényi differential privacy profile.
    ///
    /// The callback must return an upper bound on the mechanism's RDP epsilon
    /// at Rényi order `alpha`.
    ///
    /// # Why honest-but-curious?
    ///
    /// The callback should implement a well-defined RDP curve:
    ///
    /// * is functionally pure
    /// * returns a finite or infinite non-negative value.
    /// * returns an upper bound on the true RDP value at order `alpha`.
    ///
    /// The supplied profile is a valid RDP profile, not just pointwise numbers.
    /// In particular, it should satisfy the usual RDP regularity/convexity
    /// structure needed by RDP-to-DP conversions.
    #[cfg(feature = "honest-but-curious")]
    #[allow(non_snake_case)]
    pub fn with_renyiDP(
        mut self,
        curve: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.renyi_dp = Some(Arc::new(curve));
        Ok(self)
    }

    #[allow(non_snake_case)]
    pub(crate) fn with_renyiDP_trusted(
        mut self,
        curve: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.renyi_dp = Some(Arc::new(curve));
        Ok(self)
    }

    /// Construct a privacy curve from a zero-concentrated differential privacy
    /// parameter `rho`.
    #[allow(non_snake_case)]
    pub fn with_zCDP(mut self, rho: f64) -> Fallible<Self> {
        if rho.is_nan() {
            return fallible!(FailedMap, "rho must not be NaN");
        }

        if rho.is_sign_negative() {
            return fallible!(FailedMap, "rho ({}) must be non-negative", rho);
        }

        self.zcdp = Some(rho);
        Ok(self)
    }

    /// Evaluate the privacy profile at `epsilon`.
    ///
    /// # Arguments
    /// * `epsilon` - What to fix epsilon to compute delta.
    fn delta_base(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;

        let delta = if let Some(Profile { delta, scale }) = &self.profile {
            delta_via_profile(delta.as_ref(), *scale, epsilon)
        } else if let Some(points) = &self.approx_dp {
            delta_via_approxDP(points, epsilon)
        } else if let Some(Tradeoff { beta, symmetric }) = &self.tradeoff {
            delta_via_tradeoff(beta.as_ref(), *symmetric, epsilon)
        } else if let Some(mu) = &self.gaussian_dp {
            delta_via_gaussianDP(*mu, epsilon)
        } else if let Some(rho) = &self.zcdp {
            delta_via_zCDP(*rho, epsilon)
        } else if let Some(curve) = &self.renyi_dp {
            delta_via_renyiDP(curve.as_ref(), epsilon)
        } else {
            return fallible!(FailedFunction, "PrivacyCurve has no representation");
        }?;

        check_delta(delta)?;
        Ok(delta)
    }

    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        let mut delta = self.delta_base(epsilon)?;

        if self.delta_slack > 0.0 {
            delta = (delta + self.delta_slack).next_up().clamp(0.0, 1.0);
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

        let beta = if let Some(mu) = &self.gaussian_dp {
            beta_via_gaussianDP(*mu, alpha)?
        } else if let Some(Tradeoff { beta, .. }) = &self.tradeoff {
            beta(alpha)?
        } else if let Some(Profile { delta, scale }) = &self.profile {
            beta_via_profile(delta.as_ref(), *scale, alpha)?
        } else if let Some(points) = &self.approx_dp {
            beta_via_approxDP(points, alpha)?
        } else if let Some(rho) = &self.zcdp {
            beta_via_zCDP(*rho, alpha)?
        } else if let Some(curve) = &self.renyi_dp {
            beta_via_renyiDP(curve.as_ref(), alpha)?
        } else {
            return fallible!(FailedFunction, "PrivacyCurve has no representation");
        };

        check_beta(beta)?;
        Ok(beta)
    }

    pub fn beta(&self, alpha: f64) -> Fallible<f64> {
        if self.delta_slack == 0.0 {
            return self.beta_base(alpha);
        }
        let curve = self.clone();
        // TODO: this could be pushed deeper into the calculation for efficiency
        beta_via_profile(
            &move |epsilon| curve.delta(epsilon),
            ProfileScale::Delta,
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

        if delta < self.delta_slack {
            return Ok(f64::INFINITY);
        }
        let remaining_delta = (delta - self.delta_slack).clamp(0.0, 1.0);

        // Fast path only when ApproxDP is the preferred delta representation.
        // If profile exists, self.delta(...) would use profile, so do not bypass it.
        if self.profile.is_none() {
            if let Some(points) = &self.approx_dp {
                return epsilon_via_approxdp(points, remaining_delta);
            }
        }

        let mut e_min: f64 = 0.0;
        let mut e_max: f64 = 2.0;

        // Exponential search for an upper bracket.
        loop {
            let d_max = self.delta(e_max)?;

            if !d_max.is_finite() {
                return fallible!(FailedMap, "delta(epsilon) returned a non-finite value");
            }

            if d_max <= delta {
                break;
            }

            e_min = e_max;

            // Useful to have fast growth for when epsilon should be infinite.
            let next = e_max * e_max;

            if !next.is_finite() || next <= e_max {
                return Ok(f64::INFINITY);
            }

            e_max = next;
        }

        // Binary search for the smallest certified epsilon.
        loop {
            let e_mid = e_min + (e_max - e_min) / 2.0;

            if e_mid == e_min || e_mid == e_max {
                let d_min = self.delta(e_min)?;

                if !d_min.is_finite() {
                    return fallible!(FailedMap, "delta(epsilon) returned a non-finite value");
                }

                // Usually e_max is the certified endpoint. But if e_min is
                // already certified, return it to avoid ugly one-ulp artifacts.
                return Ok(if d_min <= delta { e_min } else { e_max });
            }

            let d_mid = self.delta(e_mid)?;

            if !d_mid.is_finite() {
                return fallible!(FailedMap, "delta(epsilon) returned a non-finite value");
            }

            if d_mid > delta {
                // Not yet certified sufficient.
                e_min = e_mid;
            } else {
                // Certified sufficient.
                e_max = e_mid;
            }
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

        let alpha = if let Some(mu) = &self.gaussian_dp {
            beta_via_gaussianDP(*mu, beta)?
        } else if let Some(Tradeoff {
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
        } else if let Some(Profile { delta, scale }) = &self.profile {
            beta_via_profile(delta.as_ref(), *scale, beta)?
        } else if let Some(points) = &self.approx_dp {
            beta_via_approxDP(points, beta)?
        } else if let Some(rho) = &self.zcdp {
            beta_via_zCDP(*rho, beta)?
        } else if let Some(curve) = &self.renyi_dp {
            beta_via_renyiDP(curve.as_ref(), beta)?
        } else {
            return fallible!(FailedFunction, "PrivacyCurve has no representation");
        };

        check_alpha(alpha)?;
        Ok(alpha)
    }

    fn alpha_by_inverting_beta(&self, beta: f64) -> Fallible<f64> {
        let mut a_min = 0f64;
        let mut a_max = 1.0;

        loop {
            let a_mid = a_min + (a_max - a_min) / 2.0;

            if a_mid == a_min || a_mid == a_max {
                // a_min is the conservative lower side of the crossing.
                return Ok(a_min.clamp(0.0, 1.0));
            }

            let b_mid = self.beta(a_mid)?;

            if !b_mid.is_finite() {
                return fallible!(FailedMap, "beta(alpha) returned a non-finite value");
            }

            if b_mid > beta {
                // Need larger alpha to drive beta(alpha) down.
                a_min = a_mid;
            } else {
                a_max = a_mid;
            }
        }
    }

    pub(crate) fn compose(curves: Vec<Self>) -> Fallible<Self> {
        let delta_slack = curves.iter().try_fold(0.0, |acc, curve| {
            check_delta(curve.delta_slack)?;
            Fallible::Ok((acc + curve.delta_slack).min(1.0))
        })?;

        let mut out = PrivacyCurve::new().with_delta_slack(delta_slack)?;
        let mut composed_any_base_repr = false;

        if let Some(mu) = compose_gaussianDP(&curves)? {
            out.gaussian_dp = Some(mu);
            composed_any_base_repr = true;
        }

        if curves.iter().any(|curve| curve.renyi_dp.is_some()) {
            if let Some(renyi_dp) = compose_renyiDP_with_zCDP_normalization(&curves)? {
                out.renyi_dp = Some(renyi_dp);
                composed_any_base_repr = true;
            }
        } else if let Some(rho) = compose_zCDP(&curves)? {
            out.zcdp = Some(rho);
            composed_any_base_repr = true;
        }

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

        Ok(out)
    }

    fn has_base_repr(&self) -> bool {
        self.approx_dp.is_some()
            || self.gaussian_dp.is_some()
            || self.profile.is_some()
            || self.tradeoff.is_some()
            || self.renyi_dp.is_some()
            || self.zcdp.is_some()
    }
}

fn compose_gaussianDP(curves: &[PrivacyCurve]) -> Fallible<Option<f64>> {
    let mut sum_mu2 = 0.0;
    let mut saw_non_identity = false;

    for curve in curves {
        match curve.gaussian_dp {
            Some(mu) => {
                check_mu(mu)?;
                sum_mu2 += mu * mu;
                saw_non_identity = true;
            }
            None if !curve.has_base_repr() => {}
            None => return Ok(None),
        }
    }

    Ok(saw_non_identity.then(|| sum_mu2.sqrt()))
}

fn compose_zCDP(curves: &[PrivacyCurve]) -> Fallible<Option<f64>> {
    let mut rho_sum = 0.0;
    let mut saw_non_identity = false;

    for curve in curves {
        match curve.zcdp {
            Some(rho) => {
                check_rho(rho)?;
                rho_sum += rho;
                saw_non_identity = true;
            }
            None if !curve.has_base_repr() => {}
            None => return Ok(None),
        }
    }

    Ok(saw_non_identity.then_some(rho_sum))
}

#[derive(Clone)]
enum RdpComponent {
    RenyiDP(Arc<RenyiFn>),
    ZCDP(f64),
    Zero,
}

fn compose_renyiDP_with_zCDP_normalization(
    curves: &[PrivacyCurve],
) -> Fallible<Option<Arc<RenyiFn>>> {
    let mut components = Vec::with_capacity(curves.len());
    let mut saw_non_identity = false;

    for curve in curves {
        if let Some(renyi_dp) = &curve.renyi_dp {
            components.push(RdpComponent::RenyiDP(renyi_dp.clone()));
            saw_non_identity = true;
        } else if let Some(rho) = curve.zcdp {
            check_rho(rho)?;
            components.push(RdpComponent::ZCDP(rho));
            saw_non_identity = true;
        } else if !curve.has_base_repr() {
            components.push(RdpComponent::Zero);
        } else {
            return Ok(None);
        }
    }

    if !saw_non_identity {
        return Ok(None);
    }

    Ok(Some(Arc::new(move |alpha: f64| -> Fallible<f64> {
        check_renyi_order(alpha)?;

        components.iter().try_fold(0.0, |sum, component| {
            let eps = match component {
                RdpComponent::RenyiDP(curve) => curve(alpha)?,
                RdpComponent::ZCDP(rho) => alpha * rho,
                RdpComponent::Zero => 0.0,
            };

            if eps.is_nan() || eps < 0.0 {
                return fallible!(
                    FailedMap,
                    "RDP epsilon ({eps}) must be non-negative and not NaN"
                );
            }

            Ok(sum + eps)
        })
    })))
}

fn compose_singleton_approxDP(curves: &[PrivacyCurve]) -> Fallible<Option<Arc<[ApproxDPPoint]>>> {
    let mut epsilon_sum = 0.0;
    let mut delta_sum = 0.0;
    let mut saw_non_identity = false;

    for curve in curves {
        match curve.approx_dp.as_deref() {
            Some([point]) => {
                epsilon_sum += point.epsilon;
                delta_sum = (delta_sum + point.delta).min(1.0);
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

    Ok(Some(Arc::from(
        vec![ApproxDPPoint::build((epsilon_sum, delta_sum))?].into_boxed_slice(),
    )))
}

fn check_rho(rho: f64) -> Fallible<()> {
    if rho.is_nan() {
        return fallible!(FailedMap, "rho must not be NaN");
    }
    if rho.is_sign_negative() {
        return fallible!(FailedMap, "rho ({rho}) must be non-negative");
    }
    Ok(())
}

fn check_renyi_order(alpha: f64) -> Fallible<()> {
    if !alpha.is_finite() || alpha <= 1.0 {
        return fallible!(
            FailedMap,
            "Rényi order alpha ({alpha}) must be finite and greater than one"
        );
    }
    Ok(())
}

fn check_epsilon(epsilon: f64) -> Fallible<()> {
    if epsilon.is_nan() {
        return fallible!(FailedMap, "epsilon must not be nan");
    }
    if epsilon < 0.0 {
        return fallible!(
            FailedMap,
            "epsilon ({epsilon}) must be a non-negative number"
        );
    }
    Ok(())
}
fn check_mu(mu: f64) -> Fallible<()> {
    if mu.is_nan() {
        return fallible!(FailedMap, "mu must not be nan");
    }
    if !mu.is_finite() || mu < 0.0 {
        return fallible!(FailedMap, "mu ({mu}) must be a finite non-negative number");
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
    if !(0.0..=1.0).contains(&value) {
        return fallible!(FailedMap, "{name} ({value}) must be between zero and one");
    }
    Ok(())
}
