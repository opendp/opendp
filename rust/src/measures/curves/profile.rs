use crate::{error::Fallible, measures::curves::LogProfileFn};

use super::{eval_log_profile, logspace::delta_from_log_upper_unchecked};

/// Evaluate a conservative upper bound on a log-delta privacy profile.
pub fn delta_via_profile(profile: &LogProfileFn, epsilon: f64) -> Fallible<f64> {
    Ok(delta_from_log_upper_unchecked(eval_log_profile(profile, epsilon)?)?.upper_f64()?)
}
