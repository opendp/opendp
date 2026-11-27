use super::*;
use crate::measures::zcdp::zcdp_delta;

use crate::error::ErrorVariant;
use crate::{error::Fallible, traits::Float};

pub(crate) fn cdp_epsilon<Q: Float>(rho: Q, delta: Q) -> Fallible<Q> {
    if rho < Q::zero() {
        return fallible!(FailedMap, "rho ({}) must be non-negative", rho);
    }

    if delta < Q::zero() {
        return fallible!(FailedMap, "delta ({}) must be non-negative", delta);
    }

    if rho.is_zero() {
        return Ok(Q::zero());
    }

    if delta.is_zero() {
        return Ok(Q::infinity());
    }

    if rho.is_infinite() {
        return Ok(Q::infinity());
    }

    if delta > Q::one() {
        return fallible!(FailedMap, "delta must not be greater than one");
    }

    let _1 = Q::one();
    let _2 = _1 + _1;

    // It has been proven that...
    //     delta = exp((α-1) (αρ - ε) + α ln1p(-1/α)) / (α-1)
    // ...for any choice of alpha in (1, infty)

    // The following expression is equivalent for ε:
    //   epsilon = δρ + (ln(1/δ) + (α - 1)ln(1 - 1/α) - ln(α)) / (α - 1)

    // This algorithm searches for the best alpha, the alpha that minimizes epsilon.

    // Since any alpha in (1, infty) yields a valid upper bound on epsilon,
    //    the search for alpha does not need conservative rounding.
    // If this search is slightly "incorrect" by float rounding it will only result in larger epsilon (still valid)

    // We now choose bounds for the binary search over alpha.

    // Take the derivative wrt α and check if positive:
    let deriv_pos = |a: Q| rho > -(a * delta).ln() / (a - _1).powi(2);
    //                     ρ   > -ln(αδ)           / (α - 1)^2

    // Don't let alpha be too small, due to numerical stability.
    // We only encounter α <= 1.01 when eps <= rho or close to it.
    // This is not an interesting parameter regime, as you will
    //     inherently get large delta in this regime.
    let mut a_min = Q::round_cast(1.01f64)?;

    // Find an upper bound for alpha via an exponential search
    let mut a_max = _2;
    while !deriv_pos(a_max) {
        a_max *= _2;
    }

    // run binary search to find ideal alpha
    // Since the function is convex (when restricted to the bounds)
    //     the ideal alpha is the critical point of the derivative of the function for delta
    loop {
        let diff = a_max - a_min;

        let a_mid = a_min + diff / _2;

        if a_mid == a_max || a_mid == a_min {
            break;
        }

        if deriv_pos(a_mid) {
            a_max = a_mid;
        } else {
            a_min = a_mid;
        }
    }

    // calculate epsilon

    let a_m1 = a_max.inf_sub(&_1)?;

    // reorganize 1 - 1/α -> (α-1)/α for numerical stability
    //  numer = ln(1/δ) + (α-1) ln((α-1)/α) - ln(α)
    let numer = (a_m1.inf_div(&a_max)?.inf_ln()?.inf_mul(&a_m1)?)
        .inf_sub(&a_max.inf_ln()?)?
        .inf_add(&delta.recip().inf_ln()?)?;

    //  denom = α - 1
    let denom = a_max.neg_inf_sub(&_1)?;

    //  epsilon = δρ + (ln(1/δ) + (α-1) ln((α-1)/α) - ln(α)) / (α - 1)
    //                  -----------------------------------  /  -----
    //          = δρ                          + numer        / denom
    let epsilon = a_max.inf_mul(&rho)?.inf_add(&numer.inf_div(&denom)?)?;

    Ok(epsilon.max(Q::zero()))
}

#[test]
fn test_zcdp_delta_rounding_regression() -> Fallible<()> {
    use rug::Float as MPFloat;

    for (rho, epsilon, previous) in [
        (0.05, 0.25, 8.177275310369558e-2),
        // The optimal order exceeds 2^53.
        (5e-30, 1e-13, 2.6196763297300477e-234),
    ] {
        let alpha = MPFloat::with_val(512, zcdp_order(rho, epsilon));
        let t: MPFloat = alpha.clone() - 1.0;
        let gamma = alpha.clone() * rho;
        let correction = alpha.clone() * (-MPFloat::with_val(512, 1) / alpha).ln_1p();
        let reference = (t.clone() * (gamma - epsilon) + correction - t.ln()).exp();
        let bound = zcdp_delta(rho, epsilon)?;
        assert!(MPFloat::with_val(512, bound) >= reference);
        assert!(bound <= previous * (1.0 + 1e-12));
    }
    Ok(())
}

#[test]
fn test_zcdp_direct_inverse() -> Fallible<()> {
    use rug::Float as MPFloat;

    for (rho, log_delta) in [(0.05, -5.0), (0.5, -1000.0), (1e-30, -1e6)] {
        let alpha = MPFloat::with_val(512, zcdp_inverse_order(rho, log_delta));
        let t: MPFloat = alpha.clone() - 1.0;
        let gamma = alpha.clone() * rho;
        let correction =
            alpha.clone() * (-MPFloat::with_val(512, 1) / alpha).ln_1p() - t.clone().ln();
        let reference = gamma + (correction - log_delta) / t;
        let epsilon = zcdp_epsilon_from_log_delta(rho, log_delta)?;
        assert!(MPFloat::with_val(512, epsilon) >= reference);
        assert!(zcdp_log_delta(rho, epsilon)? <= log_delta + 1e-8);
    }
    assert_eq!(zcdp_epsilon_from_log_delta(0.0, f64::NEG_INFINITY)?, 0.0);
    assert_eq!(
        zcdp_epsilon_from_log_delta(0.5, f64::NEG_INFINITY)?,
        f64::INFINITY
    );
    Ok(())
}

#[test]
fn test_numeric_errors_propagate() {
    for (rho, eps) in [(f64::NAN, 0.1), (0.1, f64::NAN)] {
        assert_eq!(
            zcdp_delta(rho, eps).unwrap_err().variant,
            ErrorVariant::NumericError
        );
    }
}

#[test]
fn test_alpha_heuristic_overflow_preserves_positive_delta() -> Fallible<()> {
    assert!(zcdp_log_delta(f64::from_bits(1), 1.0)? < -1000.0);
    let delta = zcdp_delta(0.03125, f64::MAX)?;
    // A finite epsilon has no certified pure-DP endpoint here. A rounded-up
    // subnormal is valid; silently returning mathematical zero would not be.
    assert!(delta > 0.0 && delta < f64::MIN_POSITIVE);
    Ok(())
}

#[test]
fn test_edge_cases() -> Fallible<()> {
    // Signed zero is mathematically zero and is accepted.
    assert_eq!(zcdp_delta(-0., 0.)?, 0.);
    assert_eq!(zcdp_delta(0., -0.)?, 0.);
    assert_eq!(zcdp_delta(0., 0.)?, 0.);

    let delta = zcdp_delta(0.5, 0.)?;
    assert!(delta <= 0.558835639347436);
    assert!(cdp_epsilon(0.5, delta)? <= 1e-14);

    assert!(zcdp_delta(0.1, 0.1)? > 0.);
    assert_eq!(zcdp_delta(0.1, f64::INFINITY)?, 0.);
    assert_eq!(zcdp_delta(f64::INFINITY, 1.)?, 1.0);
    assert_eq!(zcdp_log_delta(f64::MAX, 0.0)?, 0.0);
    assert_eq!(zcdp_log_delta(f64::MAX, f64::MAX)?, 0.0);

    Ok(())
}
