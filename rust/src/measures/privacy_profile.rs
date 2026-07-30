use std::{fmt, sync::Arc};

use crate::{
    error::{ErrorVariant, Fallible},
    traits::{DInterval, InfExp},
    utilities::search::{Above, fallible_binary_search},
};

type DeltaFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type LogDeltaFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type EpsilonFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

/// A privacy profile with an authoritative `epsilon -> log(delta)` map and an
/// optional independently certified inverse.
///
/// Legacy ordinary-delta callbacks are retained directly so public `delta()`
/// queries and generic inversion do not introduce a delta -> log -> delta
/// round trip. Built-in analytic profiles should prefer
/// [`PrivacyProfile::new_log_with_epsilon`].
#[derive(Clone)]
pub struct PrivacyProfile {
    delta: Option<Arc<DeltaFn>>,
    log_delta: Arc<LogDeltaFn>,
    epsilon: Option<Arc<EpsilonFn>>,
}

impl PrivacyProfile {
    /// Construct a profile from an ordinary `epsilon -> delta` callback.
    ///
    /// The callback must return an upward-conservative value in `[0, 1]`.
    pub fn new(delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        let delta: Arc<DeltaFn> = Arc::new(delta);
        let delta_for_log = delta.clone();

        Self {
            delta: Some(delta),
            log_delta: Arc::new(move |epsilon| {
                delta_to_log_upper(checked_delta(delta_for_log(epsilon)?)?)
            }),
            epsilon: None,
        }
    }

    /// Construct a profile from an authoritative `epsilon -> log(delta)` map.
    ///
    /// The callback must return an upward-conservative value in `[-infinity, 0]`.
    pub fn new_log(log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        Self {
            delta: None,
            log_delta: Arc::new(log_delta),
            epsilon: None,
        }
    }

    /// Construct a built-in profile with independently certified forward and
    /// reverse maps.
    ///
    /// This constructor is crate-private because accepting two unrelated user
    /// callbacks would create two sources of truth. A built-in caller is
    /// responsible for proving that `epsilon(delta)` is conservative for the
    /// same guarantee represented by `log_delta(epsilon)`.
    pub(crate) fn new_log_with_epsilon(
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        epsilon: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Self {
        Self {
            delta: None,
            log_delta: Arc::new(log_delta),
            epsilon: Some(Arc::new(epsilon)),
        }
    }

    /// Return a conservative upper bound on `log(delta)`.
    pub fn log_delta(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;
        checked_log_delta((self.log_delta)(epsilon)?)
    }

    /// Return a conservative upper bound on delta.
    pub fn delta(&self, epsilon: f64) -> Fallible<f64> {
        check_epsilon(epsilon)?;

        if let Some(delta) = &self.delta {
            return checked_delta(delta(epsilon)?);
        }

        log_to_delta_upper(self.log_delta(epsilon)?)
    }

    /// Return a conservative upper bound on the smallest epsilon satisfying the
    /// requested delta.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;

        if delta == 1.0 {
            return Ok(0.0);
        }

        self.epsilon_unchecked(delta)
    }

    pub(crate) fn epsilon_unchecked(&self, delta: f64) -> Fallible<f64> {
        if let Some(epsilon) = &self.epsilon {
            return checked_epsilon_output(epsilon(delta)?);
        }

        // Preserve the exact ordinary-delta comparison for compatibility
        // profiles. This avoids computing a certified logarithm at every search
        // step and avoids false rejection caused by separately rounded logs.
        if self.delta.is_some() {
            return self.invert_delta(delta);
        }

        self.invert_log_delta(delta)
    }

    fn invert_delta(&self, target_delta: f64) -> Fallible<f64> {
        if self.delta(0.0)? <= target_delta {
            return Ok(0.0);
        }

        match fallible_binary_search(
            |epsilon| Ok(self.delta(*epsilon)? <= target_delta),
            Above(0.0),
        ) {
            Ok(epsilon) => Ok(epsilon),
            Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
            Err(err) => Err(err),
        }
    }

    fn invert_log_delta(&self, target_delta: f64) -> Fallible<f64> {
        if target_delta == 0.0 {
            if self.log_delta(0.0)? == f64::NEG_INFINITY {
                return Ok(0.0);
            }

            return match fallible_binary_search(
                |epsilon| Ok(self.log_delta(*epsilon)? == f64::NEG_INFINITY),
                Above(0.0),
            ) {
                Ok(epsilon) => Ok(epsilon),
                Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
                Err(err) => Err(err),
            };
        }

        // The profile returns an upper bound on log(delta). Compare it with a
        // lower bound on the exact logarithm of the requested f64 delta:
        //
        //   log_delta_true(epsilon)
        //       <= log_delta_upper(epsilon)
        //       <= log_target_lower
        //       <= log(target_delta).
        let log_target_lower = DInterval::point(target_delta)?.ln()?.lower_f64()?;

        if self.log_delta(0.0)? <= log_target_lower {
            return Ok(0.0);
        }

        match fallible_binary_search(
            |epsilon| Ok(self.log_delta(*epsilon)? <= log_target_lower),
            Above(0.0),
        ) {
            Ok(epsilon) => Ok(epsilon),
            Err(err) if err.variant == ErrorVariant::Search => Ok(f64::INFINITY),
            Err(err) => Err(err),
        }
    }
}

impl fmt::Debug for PrivacyProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivacyProfile")
            .field("has_direct_delta", &self.delta.is_some())
            .field("has_direct_epsilon", &self.epsilon.is_some())
            .finish_non_exhaustive()
    }
}

fn delta_to_log_upper(delta: f64) -> Fallible<f64> {
    if delta == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if delta == 1.0 {
        return Ok(0.0);
    }

    DInterval::point(delta)?.ln()?.upper_f64()
}

fn log_to_delta_upper(log_delta: f64) -> Fallible<f64> {
    let log_delta = checked_log_delta(log_delta)?;

    if log_delta == f64::NEG_INFINITY {
        return Ok(0.0);
    }
    if log_delta == 0.0 {
        return Ok(1.0);
    }

    Ok(log_delta.inf_exp()?.min(1.0))
}

fn checked_delta(delta: f64) -> Fallible<f64> {
    check_delta(delta)?;
    Ok(delta)
}

fn checked_log_delta(log_delta: f64) -> Fallible<f64> {
    if log_delta.is_nan() || log_delta > 0.0 {
        return fallible!(
            FailedMap,
            "log(delta) ({log_delta}) must be at most zero and not NaN"
        );
    }
    Ok(log_delta)
}

fn checked_epsilon_output(epsilon: f64) -> Fallible<f64> {
    if epsilon.is_nan() || epsilon < 0.0 {
        return fallible!(
            FailedMap,
            "epsilon ({epsilon}) must be non-negative and not NaN"
        );
    }
    Ok(epsilon.max(0.0))
}

fn check_epsilon(epsilon: f64) -> Fallible<()> {
    if epsilon.is_nan() || epsilon.is_sign_negative() {
        return fallible!(
            FailedMap,
            "epsilon ({epsilon}) must be non-negative and not NaN"
        );
    }
    Ok(())
}

fn check_delta(delta: f64) -> Fallible<()> {
    if delta.is_nan() || delta.is_sign_negative() || delta > 1.0 {
        return fallible!(FailedMap, "delta ({delta}) must be between zero and one");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    #[test]
    fn test_legacy_delta_profile_preserves_direct_queries() -> Fallible<()> {
        let profile = PrivacyProfile::new(|epsilon| Ok((-epsilon).exp()));
        assert_eq!(profile.delta(2.0)?, (-2.0f64).exp());
        assert!(profile.log_delta(2.0)? >= -2.0);
        assert!((profile.epsilon((-2.0f64).exp())? - 2.0).abs() <= f64::EPSILON);
        Ok(())
    }

    #[test]
    fn test_log_profile_generic_inversion() -> Fallible<()> {
        let profile = PrivacyProfile::new_log(|epsilon| Ok(-epsilon));
        assert!(profile.delta(2.0)? >= (-2.0f64).exp());
        assert!(profile.epsilon((-2.0f64).exp())? >= 2.0);
        assert_eq!(profile.epsilon(0.0)?, f64::INFINITY);
        Ok(())
    }

    #[test]
    fn test_certified_inverse_bypasses_generic_search() -> Fallible<()> {
        let forward_calls = Arc::new(AtomicUsize::new(0));
        let forward_calls_ = forward_calls.clone();

        let profile = PrivacyProfile::new_log_with_epsilon(
            move |epsilon| {
                forward_calls_.fetch_add(1, Ordering::Relaxed);
                Ok(-epsilon)
            },
            |_delta| Ok(3.0),
        );

        assert_eq!(profile.epsilon(1e-6)?, 3.0);
        assert_eq!(forward_calls.load(Ordering::Relaxed), 0);
        Ok(())
    }

    #[test]
    fn test_validation() {
        let profile = PrivacyProfile::new_log(|_| Ok(0.0));
        assert!(profile.delta(-0.0).is_err());
        assert!(profile.epsilon(-0.0).is_err());

        let invalid = PrivacyProfile::new_log(|_| Ok(0.1));
        assert!(invalid.delta(0.0).is_err());
    }
}
