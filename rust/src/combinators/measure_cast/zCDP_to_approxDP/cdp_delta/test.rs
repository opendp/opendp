use super::*;

use crate::{error::Fallible, traits::Float};

pub(crate) fn cdp_epsilon<Q: Float>(rho: Q, delta: Q) -> Fallible<Q> {
    if rho.is_sign_negative() {
        return fallible!(FailedMap, "rho ({}) must be non-negative", rho);
    }

    if delta.is_sign_negative() {
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
fn test_edge_cases() -> Fallible<()> {
    // negativity checks
    assert!(cdp_delta(-0., 0.).is_err());
    assert!(cdp_delta(0., -0.).is_err());

    assert_eq!(cdp_delta(0., 0.)?, 0.);

    let delta = cdp_delta(0.5, 0.)?;
    assert_eq!(delta, 0.5588356393474351);
    assert_eq!(cdp_epsilon(0.5, delta)?, 0.0);

    assert!(cdp_delta(0.1, 0.1)? > 0.);
    assert_eq!(cdp_delta(0.1, f64::INFINITY)?, 0.);
    assert_eq!(cdp_delta(f64::INFINITY, 1.)?, 1.0);

    Ok(())
}

/// delta must round UP at the chosen alpha; the alpha search itself needs no
/// directed rounding, since a suboptimal alpha only loosens the upper bound.
fn check_cdp_delta_rounding_direction(rho: f64, eps: f64) -> Fallible<f64> {
    use crate::test_rounding::{Interval, assert_rounds_up};
    use dashu::rational::RBig;
    use std::ops::Neg;

    // the implementation's binary search for alpha, verbatim
    let mut a_max = eps
        .inf_add(&1.0)?
        .inf_div(&(2.0).neg_inf_mul(&rho)?)?
        .inf_add(&2.0)?;
    let mut a_min = 1.01f64;
    loop {
        let diff = a_max - a_min;
        let a_mid = a_min + diff / 2.0;
        if a_mid == a_max || a_mid == a_min {
            break;
        }
        let deriv = (2.0 * a_mid - 1.0) * rho - eps + a_mid.recip().neg().ln_1p();
        if deriv.is_sign_negative() {
            a_min = a_mid;
        } else {
            a_max = a_mid;
        }
    }
    let a = a_max;

    // a-1 in rationals: the f64 a - 1.0 is inexact for a >= 2^53
    let ra = RBig::try_from(a)?;
    let ra_1 = ra.clone() - RBig::ONE;
    let i_a = Interval::from_rational(&ra);
    let i_a_1 = Interval::from_rational(&ra_1);

    // t1 = (a-1)(a rho - eps), exact; t2 = a ln((a-1)/a)
    let t1 = ra_1.clone() * (ra.clone() * RBig::try_from(rho)? - RBig::try_from(eps)?);
    let t2 = i_a_1.div(&i_a).ln().mul(&i_a);
    let truth = Interval::from_rational(&t1).add(&t2).exp().div(&i_a_1);

    let delta = cdp_delta(rho, eps)?;

    // direction-mirrored assembly at the same alpha; ar_e < 0 in both
    // parameterizations, so the mirror of the (a-1) rounding is inf_sub
    let ar_e = a.neg_inf_mul(&rho)?.neg_inf_sub(&eps)?;
    assert!(ar_e.is_sign_negative());
    let a_1_bad = a.inf_sub(&1.0)?;
    let t1_bad = a_1_bad.neg_inf_mul(&ar_e)?;
    let t2_bad = a.neg_inf_mul(&(1.0).inf_div(&a)?.neg().neg_inf_ln_1p()?)?;
    let delta_bad = t1_bad
        .neg_inf_add(&t2_bad)?
        .neg_inf_exp()?
        .neg_inf_div(&a_1_bad)?;

    assert_rounds_up(delta, delta_bad, &truth);
    Ok(a)
}

#[test]
fn test_cdp_delta_rounding_direction() -> Fallible<()> {
    let a = check_cdp_delta_rounding_direction(0.05, 1.0)?;
    // a-1 is exact in this regime
    assert!(a < 2f64.powi(53));
    Ok(())
}

/// Tiny rho pushes alpha above 2^53, where a - 1.0 itself rounds. Ulp-slack can
/// absorb a wrong (a-1) here, so this pins the regime, not the direction.
#[test]
fn test_cdp_delta_rounding_direction_extreme_alpha() -> Fallible<()> {
    let a = check_cdp_delta_rounding_direction(5e-30, 1e-13)?;
    assert!(a >= 2f64.powi(53));
    Ok(())
}
