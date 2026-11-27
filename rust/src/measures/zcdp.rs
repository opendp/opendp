use opendp_derive::proven;

use crate::{error::Fallible, traits::CInterval};

use super::rdp_to_approxdp::linear::zcdp_log_delta;

/// Return a conservative approximate-DP delta bound for rho-zCDP.
///
/// The implementation is the existing fixed-order zCDP theorem: the search
/// chooses an order, while directed arithmetic keeps the returned delta an
/// upper bound.
#[proven(proof_path = "measures/zcdp/zcdp_delta.tex")]
pub(crate) fn zcdp_delta(rho: f64, epsilon: f64) -> Fallible<f64> {
    super::curves::logspace::log_to_delta_upper(zcdp_log_delta(rho, epsilon)?, 0.0)
}

/// Conservatively combine a converted approximate-DP delta with the source
/// delta carried by its input representation. Callers retain the semantic
/// distinction between RDP and zCDP source deltas; their arithmetic is shared.
pub(crate) fn add_source_delta(conversion_delta: f64, source_delta: f64) -> Fallible<f64> {
    check_delta(conversion_delta)?;
    check_delta(source_delta)?;

    if conversion_delta == 0.0 {
        return Ok(source_delta);
    }
    if source_delta == 0.0 {
        return Ok(conversion_delta);
    }

    Ok(
        (CInterval::point(conversion_delta)? + CInterval::point(source_delta)?)?
            .upper_f64()?
            .min(1.0),
    )
}

fn check_delta(delta: f64) -> Fallible<()> {
    if delta.is_nan() || delta < 0.0 || delta > 1.0 {
        return fallible!(FailedMap, "delta ({delta}) must be between zero and one");
    }
    Ok(())
}
