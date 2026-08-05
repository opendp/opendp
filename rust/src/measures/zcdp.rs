use num::Zero;
use opendp_derive::proven;
use std::ops::Neg;

use crate::{
    error::Fallible,
    traits::{CInterval, InfAdd, InfDiv, InfExp, InfLn1P, InfMul, InfSub},
};

#[cfg(test)]
pub(crate) mod test;

/// Return a conservative approximate-DP delta bound for rho-zCDP.
///
/// The implementation is the existing fixed-order zCDP theorem: the search
/// chooses an order, while directed arithmetic keeps the returned delta an
/// upper bound.
#[proven(proof_path = "measures/zcdp/zcdp_delta.tex")]
pub(crate) fn zcdp_delta(rho: f64, epsilon: f64) -> Fallible<f64> {
    if rho.is_sign_negative() {
        return fallible!(FailedMap, "rho ({}) must be non-negative", rho);
    }
    if epsilon.is_sign_negative() {
        return fallible!(FailedMap, "epsilon ({}) must be non-negative", epsilon);
    }
    if rho.is_zero() || epsilon.is_infinite() {
        return Ok(0.0);
    }
    if rho.is_infinite() {
        return Ok(1.0);
    }

    let mut alpha_max = epsilon
        .inf_add(&1.0)?
        .inf_div(&(2.0).neg_inf_mul(&rho)?)?
        .inf_add(&2.0)?;
    let mut alpha_min = 1.01;

    loop {
        let difference = alpha_max - alpha_min;
        let alpha_mid = alpha_min + difference / 2.0;
        if alpha_mid == alpha_max || alpha_mid == alpha_min {
            break;
        }

        let derivative = (2.0 * alpha_mid - 1.0) * rho - epsilon + alpha_mid.recip().neg().ln_1p();
        if derivative.is_sign_negative() {
            alpha_min = alpha_mid;
        } else {
            alpha_max = alpha_mid;
        }
    }

    let alpha_rho_minus_epsilon = alpha_max.inf_mul(&rho)?.inf_sub(&epsilon)?;
    let alpha_minus_one = if alpha_rho_minus_epsilon.is_sign_negative() {
        alpha_max.neg_inf_sub(&1.0)?
    } else {
        alpha_max.inf_sub(&1.0)?
    };
    let term_one = match alpha_minus_one.inf_mul(&alpha_rho_minus_epsilon) {
        Err(_)
            if alpha_minus_one.is_sign_negative() != alpha_rho_minus_epsilon.is_sign_negative() =>
        {
            f64::MIN
        }
        Ok(value) => value,
        error => error?,
    };
    let term_two = alpha_max.inf_mul(&(1.0).neg_inf_div(&alpha_max)?.neg().inf_ln_1p()?)?;

    Ok(term_one
        .inf_add(&term_two)?
        .inf_exp()?
        .inf_div(&(alpha_max.neg_inf_sub(&1.0)?))?
        .min(1.0))
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
    if delta.is_nan() || delta.is_sign_negative() || delta > 1.0 {
        return fallible!(FailedMap, "delta ({delta}) must be between zero and one");
    }
    Ok(())
}
