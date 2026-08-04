use std::{cmp::Ordering, sync::Arc};

use crate::{
    error::{ErrorVariant, Fallible},
    utilities::search::{Above, fallible_binary_search_by},
};

pub(crate) mod logspace;
mod profile_to_tradeoff;
mod tradeoff;

#[cfg(feature = "ffi")]
mod tradeoff_ffi;

use logspace::{
    check_delta, check_log_delta, delta_to_log_lower_unchecked, delta_to_log_upper_unchecked,
    log_to_delta_upper,
};

#[cfg(test)]
mod test;

/// A canonical privacy profile mapping epsilon to a conservative delta bound.
///
/// A profile is materialized either as normalized points or as a forward
/// epsilon-to-log-delta function. These are alternative representations of
/// one profile, not conjunctive guarantees.
#[derive(Clone)]
pub struct PrivacyProfile {
    repr: PrivacyProfileRepr,
}

/// Numeric privacy facts returned by a [`MultiDP`] privacy map.
///
/// Aggregate fields are conjunctive facts: each populated field is a valid
/// description of the same privacy relation. The representations inside a
/// [`PrivacyProfile`] remain alternative materializations of that profile.
#[derive(Clone, Default)]
pub struct PrivacyGuarantee {
    profile: Option<PrivacyProfile>,
    tradeoff: Option<TradeoffRepresentation>,
}

impl PrivacyGuarantee {
    /// Construct an empty guarantee.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a profile representation to this aggregate.
    pub(crate) fn with_profile(mut self, profile: PrivacyProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Transport a profile into the aggregate distance type.
    ///
    /// This is crate-visible because profile construction and its public APIs
    /// belong to [`PrivacyProfile`].
    pub(crate) fn from_profile(profile: PrivacyProfile) -> Self {
        Self::new().with_profile(profile)
    }

    /// Attach an f-DP tradeoff representation.
    ///
    /// The callback is assumed to be functionally pure and to return finite
    /// values in `[0, 1]` that are nonincreasing and convex on `[0, 1]`.
    /// Numerically approximate callbacks must be downward-conservative. These
    /// properties are part of the honest-but-curious callback contract and are
    /// not validated at runtime.
    #[cfg(feature = "honest-but-curious")]
    pub fn with_tradeoff(
        mut self,
        beta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.tradeoff = Some(TradeoffRepresentation {
            beta: Arc::new(beta),
            symmetric: false,
        });
        Ok(self)
    }

    /// Attach a symmetric f-DP tradeoff representation.
    ///
    /// The callback is assumed to be functionally pure and to return finite
    /// values in `[0, 1]` that are nonincreasing and convex on `[0, 1]`.
    /// Numerically approximate callbacks must be downward-conservative. Use
    /// this constructor only when the supplied tradeoff curve is genuinely
    /// symmetric; that assertion is part of the honest-but-curious callback
    /// contract and is not validated at runtime.
    #[cfg(feature = "honest-but-curious")]
    pub fn with_symmetric_tradeoff(
        mut self,
        beta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.tradeoff = Some(TradeoffRepresentation {
            beta: Arc::new(beta),
            symmetric: true,
        });
        Ok(self)
    }

    /// Return the explicitly supplied symmetric tradeoff representation.
    ///
    /// This does not derive a tradeoff view from another representation: some
    /// consumers, such as canonical noise, require the caller to attest that
    /// the supplied curve is symmetric.
    #[cfg(feature = "honest-but-curious")]
    pub(crate) fn symmetric_tradeoff(&self) -> Fallible<Arc<TradeoffFn>> {
        match &self.tradeoff {
            Some(TradeoffRepresentation {
                beta,
                symmetric: true,
            }) => Ok(beta.clone()),
            Some(TradeoffRepresentation {
                symmetric: false, ..
            }) => fallible!(
                FailedMap,
                "privacy guarantee requires an explicitly symmetric tradeoff representation"
            ),
            None => fallible!(
                FailedMap,
                "privacy guarantee requires an explicitly supplied symmetric tradeoff representation"
            ),
        }
    }

    /// Evaluate the tightest successfully available conservative delta bound.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;
        let mut best = None;
        let mut first_error = None;

        if let Some(profile) = &self.profile {
            match profile.delta(epsilon) {
                Ok(delta) => best = Some(delta),
                Err(error) => first_error = Some(error),
            }
        }
        if let Some(TradeoffRepresentation { beta, symmetric }) = &self.tradeoff {
            match tradeoff::delta_via_tradeoff(beta.as_ref(), *symmetric, epsilon) {
                Ok(delta) => best = Some(best.map_or(delta, |value: f64| value.min(delta))),
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        match best {
            Some(delta) => check_delta(delta).map(|_| delta),
            None => first_error.map_or_else(
                || fallible!(FailedFunction, "PrivacyGuarantee has no representation"),
                Err,
            ),
        }
    }

    /// Query the tightest successfully available conservative epsilon bound.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;
        if delta == 1.0 {
            return Ok(0.0);
        }

        let mut best = None;
        let mut first_error = None;
        if let Some(profile) = &self.profile {
            match profile.epsilon(delta) {
                Ok(epsilon) => best = Some(epsilon),
                Err(error) => first_error = Some(error),
            }
        }
        if let Some(TradeoffRepresentation { beta, symmetric }) = &self.tradeoff {
            match invert_decreasing_callback(
                |epsilon| tradeoff::delta_via_tradeoff(beta.as_ref(), *symmetric, epsilon),
                delta,
            ) {
                Ok(epsilon) => best = Some(best.map_or(epsilon, |value: f64| value.min(epsilon))),
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        match best {
            Some(epsilon) => Ok(epsilon),
            None => first_error.map_or_else(
                || fallible!(FailedFunction, "PrivacyGuarantee has no representation"),
                Err,
            ),
        }
    }

    /// Return the strongest successfully available conservative lower bound on
    /// `beta(alpha)`.
    pub fn beta(&self, alpha: f64) -> Fallible<f64> {
        check_alpha(alpha)?;
        if alpha == 1.0 {
            return Ok(0.0);
        }

        let mut best = None;
        let mut first_error = None;
        if let Some(profile) = &self.profile {
            match profile_to_tradeoff::beta_via_profile(profile, alpha) {
                Ok(beta) => best = Some(beta),
                Err(error) => first_error = Some(error),
            }
        }
        if let Some(tradeoff) = &self.tradeoff {
            match (tradeoff.beta)(alpha).and_then(|beta| check_beta(beta).map(|_| beta)) {
                Ok(beta) => best = Some(best.map_or(beta, |value: f64| value.max(beta))),
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        match best {
            Some(beta) => check_beta(beta).map(|_| beta),
            None => first_error.map_or_else(
                || fallible!(FailedFunction, "PrivacyGuarantee has no representation"),
                Err,
            ),
        }
    }

    /// Return the strongest successfully available conservative lower bound on
    /// `alpha(beta)`.
    pub fn alpha(&self, beta: f64) -> Fallible<f64> {
        check_beta(beta)?;
        if beta == 1.0 {
            return Ok(0.0);
        }

        let mut best = None;
        let mut first_error = None;
        if let Some(profile) = &self.profile {
            match invert_decreasing_callback(
                |alpha| profile_to_tradeoff::beta_via_profile(profile, alpha),
                beta,
            ) {
                Ok(alpha) => best = Some(alpha),
                Err(error) => first_error = Some(error),
            }
        }
        if let Some(TradeoffRepresentation { beta: beta_fn, .. }) = &self.tradeoff {
            match invert_beta_callback(beta_fn.as_ref(), beta) {
                Ok(alpha) => best = Some(best.map_or(alpha, |value: f64| value.max(alpha))),
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        match best {
            Some(alpha) => check_alpha(alpha).map(|_| alpha),
            None => first_error.map_or_else(
                || fallible!(FailedFunction, "PrivacyGuarantee has no representation"),
                Err,
            ),
        }
    }

    pub(crate) fn profile(&self) -> Option<&PrivacyProfile> {
        self.profile.as_ref()
    }
}

impl std::fmt::Debug for PrivacyGuarantee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivacyGuarantee")
            .field("profile", &self.profile.is_some())
            .field("tradeoff", &self.tradeoff.is_some())
            .finish()
    }
}

#[derive(Clone)]
struct TradeoffRepresentation {
    beta: Arc<TradeoffFn>,
    symmetric: bool,
}

type TradeoffFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

#[derive(Clone)]
enum PrivacyProfileRepr {
    Points(Arc<[ApproxDPPoint]>),
    Function {
        log_delta: Arc<LogProfileFn>,
        epsilon: Option<Arc<EpsilonFn>>,
    },
}

type DeltaFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
pub(crate) type LogProfileFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type EpsilonFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

#[derive(Clone, Debug)]
pub(crate) struct ApproxDPPoint {
    epsilon: f64,
    delta: f64,
}

impl PrivacyProfile {
    /// Construct a profile from a callback mapping epsilon to delta.
    ///
    /// The callback must be functionally pure, nonincreasing, and return
    /// values in `[0, 1]`. When numerically approximate, it must be an
    /// upward-conservative delta bound. These properties are not validated at
    /// runtime beyond pointwise range checks.
    pub fn new(delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        Self {
            repr: PrivacyProfileRepr::Function {
                log_delta: Arc::new(move |epsilon| {
                    let delta = eval_delta_profile(&delta, epsilon)?;
                    delta_to_log_upper_unchecked(delta)
                }),
                epsilon: None,
            },
        }
    }

    /// Construct an (epsilon, delta)-DP profile from epsilon-delta pairs.
    ///
    /// Inputs may be unsorted, repeated, plateaued, or dominated. They are
    /// validated and canonicalized into a canonical conservative step profile.
    /// The retained points define a step upper bound: delta is `1` before the
    /// first retained epsilon and is the most recent retained delta thereafter.
    /// A non-tight step profile is still a valid privacy bound; non-tightness
    /// does not make the supplied points invalid. In particular, even points
    /// sampled from an exact profile become a staircase that need not preserve
    /// convexity of `t -> delta(log(t))`.
    pub fn with_approxDP(mut self, mut points: Vec<(f64, f64)>) -> Fallible<Self> {
        if points.is_empty() {
            return fallible!(
                FailedMap,
                "privacy profile must be defined by at least one approximate-DP pair"
            );
        }

        for (epsilon, delta) in &points {
            check_epsilon(*epsilon)?;
            if !epsilon.is_finite() {
                return fallible!(
                    FailedMap,
                    "epsilon values in privacy profile must be finite"
                );
            }
            check_delta(*delta)?;
        }

        points.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));

        let mut canonical: Vec<ApproxDPPoint> = Vec::with_capacity(points.len());
        for (epsilon, delta) in points {
            let epsilon = crate::traits::CInterval::point(epsilon)?.upper_f64()?;
            if let Some(last) = canonical.last_mut() {
                if last.epsilon == epsilon {
                    // At a repeated epsilon, retain the tightest bound.
                    last.delta = last.delta.min(delta);
                    continue;
                }
                // A point that is no tighter than the preceding point is
                // dominated (including plateau points) and can be removed.
                if delta >= last.delta {
                    continue;
                }
            }
            canonical.push(ApproxDPPoint { epsilon, delta });
        }

        self.repr = PrivacyProfileRepr::Points(Arc::from(canonical.into_boxed_slice()));
        Ok(self)
    }

    /// Attach a delta-space callback, converting it to the canonical forward
    /// epsilon-to-log-delta representation.
    ///
    /// The callback must be functionally pure, nonincreasing, and return
    /// values in `[0, 1]`. When numerically approximate, it must be an
    /// upward-conservative delta bound. These properties are not validated at
    /// runtime beyond pointwise range checks.
    #[cfg(feature = "honest-but-curious")]
    pub fn with_profile(
        mut self,
        delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        let delta: Arc<DeltaFn> = Arc::new(delta);
        self.repr = PrivacyProfileRepr::Function {
            log_delta: log_delta_from_delta(delta),
            epsilon: None,
        };
        Ok(self)
    }

    /// Attach a callback already expressed as epsilon-to-log-delta.
    ///
    /// The callback must be functionally pure and nonincreasing, and return a
    /// valid log-delta bound. When numerically approximate, it must be
    /// upward-conservative. These properties are not validated at runtime
    /// beyond pointwise range checks.
    #[cfg(feature = "honest-but-curious")]
    pub fn with_log_profile(
        mut self,
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.repr = PrivacyProfileRepr::Function {
            log_delta: Arc::new(log_delta),
            epsilon: None,
        };
        Ok(self)
    }

    /// Attach a forward profile and an independently supplied inverse.
    #[cfg(feature = "honest-but-curious")]
    pub(crate) fn with_log_profile_with_epsilon(
        mut self,
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        epsilon: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.repr = PrivacyProfileRepr::Function {
            log_delta: Arc::new(log_delta),
            epsilon: Some(Arc::new(epsilon)),
        };
        Ok(self)
    }

    /// Evaluate delta(epsilon), conservatively rounded upward.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;
        match &self.repr {
            PrivacyProfileRepr::Points(points) => {
                let idx = points.partition_point(|point| point.epsilon <= epsilon);
                Ok(if idx == 0 { 1.0 } else { points[idx - 1].delta })
            }
            PrivacyProfileRepr::Function { log_delta, .. } => {
                let log_delta = eval_log_profile(log_delta.as_ref(), epsilon)?;
                log_to_delta_upper(log_delta)
            }
        }
    }

    /// Evaluate epsilon(delta), lazily inverting the forward profile when an
    /// independently supplied inverse is unavailable.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;
        if delta == 1.0 {
            return Ok(0.0);
        }

        match &self.repr {
            PrivacyProfileRepr::Points(points) => Ok(points
                .iter()
                .find(|point| point.delta <= delta)
                .map(|point| point.epsilon)
                .unwrap_or(f64::INFINITY)),
            PrivacyProfileRepr::Function { log_delta, epsilon } => {
                if let Some(epsilon) = epsilon {
                    let value = epsilon(delta)?;
                    if value.is_nan() || value < 0.0 {
                        return fallible!(
                            FailedMap,
                            "epsilon ({value}) must be non-negative and not NaN"
                        );
                    }
                    Ok(value.max(0.0))
                } else {
                    invert_log_profile(log_delta.as_ref(), delta)
                }
            }
        }
    }

    /// Return the finite epsilon endpoint at which the profile is pure DP.
    pub fn pure_epsilon(&self) -> Fallible<Option<f64>> {
        let epsilon = match &self.repr {
            PrivacyProfileRepr::Points(points) => points
                .iter()
                .find(|point| point.delta == 0.0)
                .map(|point| point.epsilon),
            PrivacyProfileRepr::Function { .. } => {
                let epsilon = self.epsilon(0.0)?;
                epsilon.is_finite().then_some(epsilon)
            }
        };
        Ok(epsilon)
    }

    /// Evaluate the canonical log-delta representation for an internal
    /// conversion without exposing how the profile is materialized.
    pub(crate) fn log_delta(&self, epsilon: f64) -> Fallible<f64> {
        match &self.repr {
            PrivacyProfileRepr::Points(_) => delta_to_log_upper_unchecked(self.delta(epsilon)?),
            PrivacyProfileRepr::Function { log_delta, .. } => {
                eval_log_profile(log_delta.as_ref(), epsilon)
            }
        }
    }
}

#[cfg(feature = "honest-but-curious")]
fn log_delta_from_delta(delta: Arc<DeltaFn>) -> Arc<LogProfileFn> {
    Arc::new(move |epsilon| {
        let value = eval_delta_profile(delta.as_ref(), epsilon)?;
        delta_to_log_upper_unchecked(value)
    })
}

fn eval_delta_profile(profile: &DeltaFn, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;
    let value = profile(epsilon)?;
    check_delta(value)?;
    Ok(value)
}

fn eval_log_profile(profile: &LogProfileFn, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;
    let value = profile(epsilon)?;
    check_log_delta(value)?;
    Ok(value)
}

fn invert_log_profile(profile: &LogProfileFn, target_delta: f64) -> Fallible<f64> {
    let callback = |epsilon| eval_log_profile(profile, epsilon);
    let target_lower = delta_to_log_lower_unchecked(target_delta)?;
    match invert_decreasing_callback(&callback, target_lower) {
        Ok(epsilon) => Ok(epsilon),
        Err(err) if err.variant == ErrorVariant::Search => {
            let target_upper = delta_to_log_upper_unchecked(target_delta)?;
            match invert_decreasing_callback(&callback, target_upper) {
                Ok(epsilon) => Ok(epsilon),
                Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

/// Invert a nonincreasing callback while preserving the left edge of plateaus.
fn invert_decreasing_callback(
    callback: impl Fn(f64) -> Fallible<f64>,
    target: f64,
) -> Fallible<f64> {
    // This caller owns the final profile quantity, so it explicitly translates
    // terminal range failures into ordering information. The generic search
    // remains literal and propagates these errors.
    let compare = |epsilon: &f64| match callback(*epsilon) {
        Ok(value) => Ok(if value <= target {
            Ordering::Less
        } else {
            Ordering::Greater
        }),
        Err(err) if err.variant == ErrorVariant::NumericRangeBelow => Ok(Ordering::Less),
        Err(err) if err.variant == ErrorVariant::NumericRangeAbove => Ok(Ordering::Greater),
        // A profile callback may overflow while evaluating a sufficiently
        // large epsilon. It is below any positive target, but it does not
        // establish a finite pure-DP endpoint when the target is exactly 0.
        Err(err) if err.variant == ErrorVariant::Overflow => Ok(if target == f64::NEG_INFINITY {
            Ordering::Greater
        } else {
            Ordering::Less
        }),
        Err(err) => Err(err),
    };

    match compare(&0.0)? {
        Ordering::Less | Ordering::Equal => return Ok(0.0),
        Ordering::Greater => {}
    }

    fallible_binary_search_by(compare, Above(0.0))
}

fn invert_beta_callback(beta: &TradeoffFn, target: f64) -> Fallible<f64> {
    invert_decreasing_callback(|alpha| beta(alpha), target)
}

fn check_alpha(alpha: f64) -> Fallible<()> {
    check_unit_interval(alpha, "alpha")
}

fn check_beta(beta: f64) -> Fallible<()> {
    check_unit_interval(beta, "beta")
}

fn check_unit_interval(value: f64, name: &str) -> Fallible<()> {
    if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
        return fallible!(FailedMap, "{name} ({value}) must be between zero and one");
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
