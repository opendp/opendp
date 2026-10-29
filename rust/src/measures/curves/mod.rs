use std::sync::Arc;

use crate::{
    error::{ErrorVariant, Fallible},
    utilities::search::{Above, fallible_binary_search},
};

pub(crate) mod logspace;
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

#[derive(Clone)]
enum PrivacyProfileRepr {
    Points(Arc<[ApproxDPPoint]>),
    Function {
        log_delta: Arc<LogProfileFn>,
        // Retain the original delta bound for exact equality comparisons.
        // A delta -> log -> delta roundtrip can otherwise lose plateaus.
        delta: Option<Arc<DeltaFn>>,
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
        let delta: Arc<DeltaFn> = Arc::new(delta);
        Self {
            repr: PrivacyProfileRepr::Function {
                log_delta: log_delta_from_delta(delta.clone()),
                delta: Some(delta),
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
            log_delta: log_delta_from_delta(delta.clone()),
            delta: Some(delta),
            epsilon: None,
        };
        Ok(self)
    }

    /// Attach a callback already expressed as epsilon-to-log-delta.
    ///
    /// The callback must be functionally pure and nonincreasing, and return a
    /// valid log-delta bound. When numerically approximate, it must be
    /// upward-conservative. These properties are not validated at runtime
    /// beyond pointwise range checks. Numerical errors are not comparison
    /// information; inversion requires an evaluable log-delta bound.
    #[cfg(feature = "honest-but-curious")]
    pub fn with_log_profile(
        mut self,
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Fallible<Self> {
        self.repr = PrivacyProfileRepr::Function {
            log_delta: Arc::new(log_delta),
            delta: None,
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
            delta: None,
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
            PrivacyProfileRepr::Function {
                delta: Some(delta), ..
            } => eval_delta_profile(delta.as_ref(), epsilon),
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
            PrivacyProfileRepr::Function {
                log_delta,
                delta: delta_profile,
                epsilon,
            } => {
                if let Some(epsilon) = epsilon {
                    let value = epsilon(delta)?;
                    if value.is_nan() || value < 0.0 {
                        return fallible!(
                            FailedMap,
                            "epsilon ({value}) must be non-negative and not NaN"
                        );
                    }
                    Ok(value.max(0.0))
                } else if let Some(profile) = delta_profile {
                    match invert_decreasing_callback(
                        |epsilon| eval_delta_profile(profile.as_ref(), epsilon),
                        delta,
                    ) {
                        Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
                        result => result,
                    }
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
}

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
        // A failed conservative comparison is not permission to weaken the
        // target. Without a certified finite endpoint, return infinity.
        Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
        Err(err) => Err(err),
    }
}

/// Invert a nonincreasing callback while preserving the left edge of plateaus.
fn invert_decreasing_callback(
    callback: impl Fn(f64) -> Fallible<f64>,
    target: f64,
) -> Fallible<f64> {
    // An overflow can originate in an intermediate operation. Neither its
    // occurrence nor its sign would establish the final log-delta comparison.
    let predicate = |epsilon: &f64| Ok(callback(*epsilon)? <= target);

    if predicate(&0.0)? {
        return Ok(0.0);
    }

    fallible_binary_search(predicate, Above(0.0))
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
