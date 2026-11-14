//! Executable Boolean randomized-response postprocessing.
//!
//! This is the *same source file* used by OpenDP's production constructor
//! and imported by the Verus harness. No reference implementation is copied.
//! The randomness provider is intentionally not part of this pure kernel.
#[cfg(verus_only)]
use vstd::prelude::*;

/// One sampled Boolean, postprocessed according to randomized response.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures result == (truth ^ !honest),
))]
pub fn release_bool(truth: bool, honest: bool) -> bool {
    truth ^ !honest
}

/// Preserve every observable sampler outcome, including its exact error.
/// The error value and the success/failure decision are never inspected
/// or modified as a function of the private truth.
#[cfg_attr(verus_only, verus_spec(result =>
    ensures result == match sampled {
        Ok(honest) => Ok(truth ^ !honest),
        Err(err) => Err(err),
    },
))]
pub fn release_result<E>(
    truth: bool,
    sampled: Result<bool, E>,
) -> Result<bool, E> {
    match sampled {
        Ok(honest) => Ok(release_bool(truth, honest)),
        Err(err) => Err(err),
    }
}
