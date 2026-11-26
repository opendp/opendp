use crate::{error::Fallible, traits::SInterval};

pub(crate) fn check_delta(delta: f64) -> Fallible<()> {
    if delta.is_nan() || delta < 0.0 || delta > 1.0 {
        return fallible!(FailedMap, "delta ({delta}) must be between zero and one");
    }
    Ok(())
}

pub(crate) fn check_log_delta(log_delta: f64) -> Fallible<()> {
    if log_delta.is_nan() || log_delta > 0.0 {
        return fallible!(
            FailedMap,
            "log(delta) ({log_delta}) must be at most zero and not NaN"
        );
    }
    Ok(())
}

/// Convert an ordinary delta upper bound to an upward-conservative log-delta
/// upper bound.
pub(in crate::measures::curves) fn delta_to_log_upper_unchecked(delta: f64) -> Fallible<f64> {
    if delta == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if delta == 1.0 {
        return Ok(0.0);
    }

    SInterval::point(delta)?.ln()?.upper_f64()
}

/// Convert an ordinary delta target to a downward-conservative log-delta
/// target for certified profile inversion.
pub(in crate::measures::curves) fn delta_to_log_lower_unchecked(delta: f64) -> Fallible<f64> {
    if delta == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if delta == 1.0 {
        return Ok(0.0);
    }

    SInterval::point(delta)?.ln()?.lower_f64()
}

/// Convert a log-delta core and ordinary source slack to an upper delta bound.
pub(crate) fn log_to_delta_upper(log_delta: f64, slack: f64) -> Fallible<f64> {
    check_log_delta(log_delta)?;
    if log_delta == f64::NEG_INFINITY {
        return Ok(slack);
    }
    if log_delta == 0.0 {
        return Ok(1.0);
    }

    // exp(-1024) is below the least positive f64. Clamping upward avoids
    // exhausting the software exponent range and still returns a positive bound.
    let core = SInterval::point(log_delta.max(-1024.0))?.exp()?;
    if slack == 0.0 {
        return core.upper_f64();
    }
    (core + SInterval::point(slack)?)?
        .min(SInterval::point(1.0)?)
        .upper_f64()
}

/// Upward-conservative log(exp(left) + exp(right)), capped at log(1).
/// Factoring out the larger argument avoids underflow of the total.
pub(in crate::measures::curves) fn logaddexp_upper(left: f64, right: f64) -> Fallible<f64> {
    if left == f64::NEG_INFINITY {
        return Ok(right);
    }
    if right == f64::NEG_INFINITY {
        return Ok(left);
    }
    let larger = left.max(right);
    let difference = (SInterval::point(left.min(right))? - SInterval::point(larger)?)?
        .max(SInterval::point(-1024.0)?);
    (SInterval::point(larger)? + difference.exp()?.ln_1p()?)?
        .min(SInterval::point(0.0)?)
        .upper_f64()
}
