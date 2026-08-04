use crate::{
    error::Fallible,
    traits::{SInterval, backend::Dashu},
};

const F64_TRUE_MIN: f64 = f64::from_bits(1);
pub(crate) const LOG_TRUE_MIN: f64 = -744.4400719213812;

pub(crate) fn check_delta(delta: f64) -> Fallible<()> {
    if delta.is_nan() || delta.is_sign_negative() || delta > 1.0 {
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

    SInterval::<Dashu>::point(delta)?.ln()?.upper_f64()
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

    SInterval::<Dashu>::point(delta)?.ln()?.lower_f64()
}

/// Convert an upward-conservative log-delta bound to an ordinary delta bound.
pub(crate) fn log_to_delta_upper(log_delta: f64) -> Fallible<f64> {
    check_log_delta(log_delta)?;
    if log_delta == f64::NEG_INFINITY {
        return Ok(0.0);
    }
    if log_delta == 0.0 {
        return Ok(1.0);
    }

    SInterval::<Dashu>::point(log_delta)?.exp()?.upper_f64()
}

/// Return a certified interval for delta from an upper log-delta bound.
pub(in crate::measures::curves) fn delta_from_log_upper_unchecked(
    log_delta: f64,
) -> Fallible<SInterval<Dashu>> {
    if log_delta == f64::NEG_INFINITY {
        return SInterval::<Dashu>::point(0.0);
    }

    let delta_upper = if log_delta < LOG_TRUE_MIN {
        F64_TRUE_MIN
    } else {
        SInterval::<Dashu>::point(log_delta)?
            .exp()?
            .upper_f64()?
            .clamp(0.0, 1.0)
    };
    SInterval::<Dashu>::between(0.0, delta_upper)
}

/// Return a certified lower bound for `1 - delta` from an upper log-delta
/// bound. The subnormal branch avoids losing the result to ordinary-f64
/// underflow or cancellation.
pub(in crate::measures::curves) fn one_minus_delta_from_log_upper_unchecked(
    log_delta: f64,
) -> Fallible<SInterval<Dashu>> {
    if log_delta == f64::NEG_INFINITY {
        return SInterval::<Dashu>::point(1.0);
    }

    if log_delta < LOG_TRUE_MIN {
        return (SInterval::<Dashu>::point(1.0)?
            - SInterval::<Dashu>::between(0.0, F64_TRUE_MIN)?)?
        .clamp01();
    }

    (SInterval::<Dashu>::point(0.0)? - SInterval::<Dashu>::point(log_delta)?.exp_m1()?)?.clamp01()
}
