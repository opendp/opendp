use crate::{error::Fallible, traits::DInterval};

/// Convert an ordinary delta upper bound to an upward-conservative log-delta
/// upper bound.
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

/// Convert an ordinary delta target to a downward-conservative log-delta
/// target for certified profile inversion.
pub(crate) fn delta_to_log_lower(delta: f64) -> Fallible<f64> {
    check_delta(delta)?;
    if delta == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if delta == 1.0 {
        return Ok(0.0);
    }

    DInterval::point(delta)?.ln()?.lower_f64()
}

/// Convert an upward-conservative log-delta bound to an ordinary delta bound.
pub(crate) fn log_to_delta_upper(log_delta: f64) -> Fallible<f64> {
    if log_delta.is_nan() || log_delta > 0.0 {
        return fallible!(
            FailedMap,
            "log(delta) ({log_delta}) must be at most zero and not NaN"
        );
    }
    if log_delta == f64::NEG_INFINITY {
        return Ok(0.0);
    }
    if log_delta == 0.0 {
        return Ok(1.0);
    }

    DInterval::point(log_delta)?.exp()?.upper_f64()
}

fn check_delta(delta: f64) -> Fallible<()> {
    if delta.is_nan() || delta.is_sign_negative() || delta > 1.0 {
        return fallible!(FailedMap, "delta ({delta}) must be between zero and one");
    }
    Ok(())
}
