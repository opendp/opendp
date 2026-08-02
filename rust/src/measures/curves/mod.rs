use core::f64;
use std::{cmp::Ordering, sync::Arc};

use crate::measures::curves::{
    approxdp::{delta_via_approxDP, epsilon_via_approxdp},
    logspace::{check_delta, delta_to_log_lower_unchecked, delta_to_log_upper_unchecked},
    profile::delta_via_profile,
};

use crate::{
    error::{ErrorVariant, Fallible},
    utilities::search::{Above, fallible_binary_search_by},
};

mod approxdp;
pub(crate) mod logspace;
mod profile;

#[cfg(test)]
mod test;

#[deprecated(since = "0.15.0", note = "Use PrivacyGuarantee instead.")]
#[allow(dead_code)] // compatibility alias is consumed by later stack commits
/// Compatibility alias while callers migrate to [`PrivacyGuarantee`].
pub type PrivacyProfile = PrivacyGuarantee;

/// Contains multiple simultaneously valid privacy representations for the same
/// mechanism and neighboring relation.
///
/// Every representation stored in a `PrivacyGuarantee` holds conjunctively.
/// Representations may differ in strength and in their closure properties under
/// later operations. Privacy-profile queries are available through
/// [`PrivacyGuarantee::delta`] and [`PrivacyGuarantee::epsilon`].
#[derive(Clone, Default)]
pub struct PrivacyGuarantee {
    // invariant: order increasing in epsilon, nonincreasing in delta
    approx_dp: Option<Arc<[ApproxDPPoint]>>,
    profile: Option<Profile>,
}

#[derive(Clone)]
struct Profile {
    // Canonical forward representation: epsilon -> log(delta).
    delta: Arc<LogProfileFn>,
    epsilon: Option<Arc<EpsilonFn>>,
}

type ProfileFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type LogProfileFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type EpsilonFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

impl Profile {
    fn new(log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        Self {
            delta: Arc::new(log_delta),
            epsilon: None,
        }
    }

    fn new_with_epsilon(
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        epsilon: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Self {
        Self {
            delta: Arc::new(log_delta),
            epsilon: Some(Arc::new(epsilon)),
        }
    }

    fn new_from_delta(delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        let delta: Arc<ProfileFn> = Arc::new(delta);
        let forward = delta.clone();

        Self {
            delta: Arc::new(move |epsilon| {
                delta_to_log_upper_unchecked(eval_delta_profile(forward.as_ref(), epsilon)?)
            }),
            epsilon: Some(Arc::new(move |target_delta| {
                invert_delta_callback(delta.as_ref(), target_delta)
            })),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ApproxDPPoint {
    epsilon: f64,
    delta: f64,
}

impl PrivacyGuarantee {
    pub fn new() -> Self {
        Default::default()
    }

    /// Construct an (ε, δ)-DP privacy profile from epsilon-delta pairs.
    ///
    /// Pairs may be supplied in any order. Every epsilon must be finite and
    /// nonnegative, and every delta must be in `[0, 1]`. After sorting by
    /// epsilon, epsilon values must be unique and delta must strictly decrease
    /// as epsilon increases. Duplicate pairs, duplicate epsilon values,
    /// repeated-delta plateaus, and dominated points are rejected rather than
    /// repaired.
    ///
    /// # Arguments
    /// * `points` - a vector of approx-DP pairs
    pub fn with_approxDP(mut self, mut points: Vec<(f64, f64)>) -> Fallible<Self> {
        if points.is_empty() {
            return fallible!(
                FailedMap,
                "privacy guarantee must be defined by at least one approximate-DP pair"
            );
        }

        // Validate every supplied point before sorting and canonical-order checks.
        // An invalid value must not be hidden by a later structural error.
        for (epsilon, delta) in &points {
            check_epsilon(*epsilon)?;
            if !epsilon.is_finite() {
                return fallible!(
                    FailedMap,
                    "epsilon values in privacy guarantee must be finite"
                );
            }
            check_delta(*delta)?;
        }

        points.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));
        for pair in points.windows(2) {
            let [(epsilon_left, delta_left), (epsilon_right, delta_right)] = pair else {
                unreachable!()
            };

            if epsilon_left == epsilon_right {
                return fallible!(
                    FailedMap,
                    "epsilon values in a privacy guarantee must be unique"
                );
            }
            if delta_left <= delta_right {
                return fallible!(
                    FailedMap,
                    "delta values must strictly decrease as epsilon increases"
                );
            }
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

    /// Attach a privacy-profile representation mapping `epsilon -> delta`.
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

    /// Attach a log-space privacy-profile representation mapping `epsilon -> log(delta)`.
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

    /// Evaluate the privacy profile at `epsilon`.
    ///
    /// # Arguments
    /// * `epsilon` - What to fix epsilon to compute delta.
    /// Return a conservative upper bound on delta at the given nonnegative epsilon.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;

        let delta = if let Some(Profile { delta: profile, .. }) = &self.profile {
            delta_via_profile(profile.as_ref(), epsilon)?
        } else if let Some(points) = &self.approx_dp {
            delta_via_approxDP(points, epsilon)?
        } else {
            return fallible!(FailedFunction, "PrivacyGuarantee has no representation");
        };

        check_delta(delta)?;
        Ok(delta)
    }

    /// Query the guarantee for the smallest `epsilon`
    /// such that `delta(epsilon) <= delta`.
    ///
    /// # Arguments
    /// * `delta` - What to fix delta to compute epsilon.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;

        if delta == 1.0 {
            return Ok(0.0);
        }

        if let Some(Profile {
            delta: profile_delta,
            epsilon,
        }) = &self.profile
        {
            if let Some(epsilon) = epsilon {
                let value = epsilon(delta)?;
                if value.is_nan() || value.is_sign_negative() {
                    return fallible!(
                        FailedMap,
                        "epsilon ({value}) must be non-negative and not NaN"
                    );
                }
                return Ok(value.max(0.0));
            }

            return invert_log_profile(profile_delta.as_ref(), delta);
        }

        if let Some(points) = &self.approx_dp {
            return epsilon_via_approxdp(points, delta);
        }

        invert_decreasing_callback(|epsilon| self.delta(epsilon), delta)
    }
}

fn invert_delta_callback(delta: &ProfileFn, target_delta: f64) -> Fallible<f64> {
    invert_decreasing_callback(|epsilon| eval_delta_profile(delta, epsilon), target_delta)
}

fn invert_log_profile(profile: &LogProfileFn, target_delta: f64) -> Fallible<f64> {
    invert_decreasing_callback(
        |epsilon| eval_log_profile(profile, epsilon),
        delta_to_log_lower_unchecked(target_delta)?,
    )
}

/// Invert a nonincreasing callback while preserving the left edge of plateaus.
///
/// `Less` denotes the passing side (`value <= target`). Terminal numeric-range
/// errors remain in the comparator result so range-aware search can order them.
fn invert_decreasing_callback(
    callback: impl Fn(f64) -> Fallible<f64>,
    target: f64,
) -> Fallible<f64> {
    let compare = |epsilon: &f64| {
        callback(*epsilon).map(|value| {
            if value <= target {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    };

    match compare(&0.0) {
        Ok(Ordering::Less | Ordering::Equal) => return Ok(0.0),
        Ok(Ordering::Greater) => {}
        Err(err) if err.variant == ErrorVariant::NumericRangeBelow => return Ok(0.0),
        Err(err) if err.variant == ErrorVariant::NumericRangeAbove => {}
        Err(err) => return Err(err),
    }

    match fallible_binary_search_by(compare, Above(0.0)) {
        Ok(epsilon) => Ok(epsilon),
        Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
        Err(err) => Err(err),
    }
}

fn eval_delta_profile(profile: &ProfileFn, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;
    let value = profile(epsilon)?;
    let value = if value == 0.0 { 0.0 } else { value };
    check_delta(value)?;
    Ok(value)
}

fn eval_log_profile(profile: &LogProfileFn, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;
    let value = profile(epsilon)?;
    if value.is_nan() || value > 0.0 {
        return fallible!(
            FailedMap,
            "log(delta) ({value}) must be at most zero and not NaN"
        );
    }
    Ok(value)
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
