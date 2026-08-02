use std::{fmt, sync::Arc};

use crate::{
    error::{ErrorVariant, Fallible},
    traits::{DInterval, InfExp},
    utilities::search::{Above, fallible_binary_search},
};

type LogDeltaFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;
type EpsilonFn = dyn Fn(f64) -> Fallible<f64> + Send + Sync;

/// A privacy profile represented canonically by an epsilon-to-log-delta map.
///
/// The optional reverse map is reserved for built-in conversions whose
/// certification is established independently of the generic inversion.
#[derive(Clone)]
pub struct PrivacyProfile {
    log_delta: Arc<LogDeltaFn>,
    epsilon: Option<Arc<EpsilonFn>>,
}

impl PrivacyProfile {
    /// Construct a profile from an upward-conservative epsilon-to-log-delta
    /// callback. The callback must return a value in `[-infinity, 0]`.
    pub fn new(log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync) -> Self {
        Self {
            log_delta: Arc::new(log_delta),
            epsilon: None,
        }
    }

    /// Construct a built-in profile with independently certified forward and
    /// reverse maps.
    ///
    /// This constructor is crate-private so arbitrary callbacks cannot claim
    /// to be trusted certified inverses.
    pub(crate) fn new_with_epsilon(
        log_delta: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
        epsilon: impl Fn(f64) -> Fallible<f64> + 'static + Send + Sync,
    ) -> Self {
        Self {
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
        log_to_delta_upper(self.log_delta(epsilon)?)
    }

    /// Return a conservative upper bound on the smallest epsilon satisfying
    /// the requested delta.
    pub fn epsilon(&self, delta: f64) -> Fallible<f64> {
        check_delta(delta)?;

        if let Some(epsilon) = &self.epsilon {
            return checked_epsilon_output(epsilon(delta)?);
        }

        self.invert_log_delta(delta)
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

        // Compare the conservative log-delta upper bound with a lower bound on
        // the exact logarithm of the requested f64 delta. This avoids an
        // extra exp/log round trip during generic inversion.
        let log_target_lower = log_delta_lower(target_delta)?;

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
            .field("has_direct_epsilon", &self.epsilon.is_some())
            .finish_non_exhaustive()
    }
}

/// Convert an ordinary delta callback result to the canonical log-delta
/// representation. This is only used by compatibility FFI/adapters.
pub(crate) fn delta_to_log_upper(delta: f64) -> Fallible<f64> {
    check_delta(delta)?;
    if delta == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if delta == 1.0 {
        return Ok(0.0);
    }

    DInterval::point(delta)?.ln()?.upper_f64()
}

fn log_delta_lower(delta: f64) -> Fallible<f64> {
    if delta == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    DInterval::point(delta)?.ln()?.lower_f64()
}

fn log_to_delta_upper(log_delta: f64) -> Fallible<f64> {
    let log_delta = checked_log_delta(log_delta)?;

    if log_delta == f64::NEG_INFINITY {
        return Ok(0.0);
    }
    if log_delta == 0.0 {
        return Ok(1.0);
    }

    // The smallest positive f64 is conservative for every smaller positive
    // result and avoids asking the arbitrary-precision backend to underflow.
    let min_positive = f64::from_bits(1);
    if log_delta <= min_positive.ln() {
        return Ok(min_positive);
    }

    Ok(log_delta.inf_exp()?.min(1.0))
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
    fn test_log_profile_generic_inversion() -> Fallible<()> {
        let profile = PrivacyProfile::new(|epsilon| Ok(-epsilon));
        assert!(profile.delta(2.0)? >= (-2.0f64).exp());
        assert!(profile.epsilon((-2.0f64).exp())? >= 2.0);
        assert_eq!(profile.epsilon(0.0)?, f64::INFINITY);
        Ok(())
    }

    #[test]
    fn test_certified_inverse_bypasses_generic_search() -> Fallible<()> {
        let forward_calls = Arc::new(AtomicUsize::new(0));
        let forward_calls_ = forward_calls.clone();

        let profile = PrivacyProfile::new_with_epsilon(
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
        let profile = PrivacyProfile::new(|_| Ok(0.0));
        assert!(profile.delta(-0.0).is_err());
        assert!(profile.epsilon(-0.0).is_err());

        let invalid = PrivacyProfile::new(|_| Ok(0.1));
        assert!(invalid.delta(0.0).is_err());
    }
}
