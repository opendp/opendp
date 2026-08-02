//! Fixed-order RDP to approximate-DP conversion kernels.
//!
//! These formulas are generic over the interval backend:
//!
//! - instantiate with `A` while selecting a promising Renyi order;
//! - instantiate with `D` for the final certified evaluation.
//!
//! The search instantiation affects only tightness. The certified instantiation
//! is the privacy-critical evaluation.
//!
//! This module assumes the interval layer provides stable implementations of:
//!
//! - `Interval::ln_1p()` for `ln(1 + x)`;
//! - `Interval::log_expm1()` for `ln(exp(x) - 1)` on positive intervals;
//! - `Interval::softplus()` for `ln(1 + exp(x))`.

use crate::{
    error::Fallible,
    traits::{Interval, IntervalArithmeticBackend, IntervalExpBackend},
};

type I<Bk> = Interval<Bk>;

/// Fixed-order Balle--Canonne upper bound on `log(delta)`.
///
/// Given an `(alpha, gamma)`-RDP guarantee, for every `epsilon >= 0`,
///
/// ```text
/// log(delta) = (alpha - 1) * (gamma - epsilon)
///            + alpha * ln(1 - 1 / alpha)
///            - ln(alpha - 1).
/// ```
///
/// The returned interval encloses the exact right-hand side.
pub(crate) fn rdp_log_delta0_on<Bk>(alpha: I<Bk>, gamma: I<Bk>, epsilon: I<Bk>) -> Fallible<I<Bk>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    check_order(&alpha)?;
    check_nonnegative(&gamma, "RDP epsilon")?;
    check_nonnegative(&epsilon, "epsilon")?;

    let one = I::<Bk>::point(1.0)?;
    let alpha_m1 = alpha.clone().sub(one.clone())?;

    // Keep this in the numerically stable form
    //
    //     alpha * ln1p(-1 / alpha) - ln(alpha - 1).
    //
    // Rewriting it as
    //
    //     (alpha - 1) ln(alpha - 1) - alpha ln(alpha)
    //
    // catastrophically cancels at large alpha and gives a bad search objective
    // near the important alpha ~= 2^53 regime.
    let neg_recip_alpha = one.div(alpha.clone())?.neg()?;
    let correction = alpha
        .clone()
        .mul(neg_recip_alpha.ln_1p()?)?
        .sub(alpha_m1.clone().ln()?)?;

    alpha_m1.mul(gamma.sub(epsilon)?)?.add(correction)
}

/// Fixed-order Asoodeh et al. upper bound on `log(delta)`.
///
/// Returns `None` when the theorem's strict `epsilon > gamma` condition cannot
/// be established with the selected interval backend.
pub(crate) fn rdp_log_delta1_on<Bk>(
    alpha: I<Bk>,
    gamma: I<Bk>,
    epsilon: I<Bk>,
) -> Fallible<Option<I<Bk>>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    check_order(&alpha)?;
    check_nonnegative(&gamma, "RDP epsilon")?;
    check_nonnegative(&epsilon, "epsilon")?;

    // The identity case should be handled by the caller because log(0) is an
    // extended-real value and Interval intentionally stores finite endpoints.
    if gamma.upper_f64()? == 0.0 {
        return Ok(None);
    }

    if epsilon.lower_f64()? <= gamma.upper_f64()? {
        return Ok(None);
    }

    let one = I::<Bk>::point(1.0)?;
    let alpha_m1 = alpha.clone().sub(one)?;
    let numerator_arg = alpha_m1.clone().mul(gamma)?;
    let denominator_arg = alpha_m1.mul(epsilon)?;

    if numerator_arg.lower_f64()? <= 0.0 || denominator_arg.lower_f64()? <= 0.0 {
        return Ok(None);
    }

    let value = numerator_arg
        .log_expm1()?
        .sub(alpha.ln()?)?
        .sub(denominator_arg.log_expm1()?)?;

    Ok(Some(value))
}

/// Fixed-order Balle--Canonne upper bound on epsilon at a requested delta.
///
/// This is the exact algebraic inverse of [`rdp_log_delta0_on`] at fixed
/// `(alpha, gamma)`, evaluated with the selected interval backend.
pub(crate) fn rdp_epsilon0_on<Bk>(alpha: I<Bk>, gamma: I<Bk>, delta: I<Bk>) -> Fallible<I<Bk>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    check_order(&alpha)?;
    check_nonnegative(&gamma, "RDP epsilon")?;
    check_probability_strictly_positive(&delta, "delta")?;

    let one = I::<Bk>::point(1.0)?;
    let zero = I::<Bk>::point(0.0)?;
    let alpha_m1 = alpha.clone().sub(one.clone())?;

    let neg_recip_alpha = one.div(alpha.clone())?.neg()?;
    let correction = alpha
        .clone()
        .mul(neg_recip_alpha.ln_1p()?)?
        .sub(alpha_m1.clone().ln()?)?;

    gamma
        .add(correction.sub(delta.ln()?)?.div(alpha_m1)?)?
        .max(zero)
}

/// Fixed-order Asoodeh et al. upper bound on epsilon at a requested delta.
///
/// Returns `None` when the theorem's strict `alpha * delta < 1` condition
/// cannot be established with the selected interval backend.
pub(crate) fn rdp_epsilon1_on<Bk>(
    alpha: I<Bk>,
    gamma: I<Bk>,
    delta: I<Bk>,
) -> Fallible<Option<I<Bk>>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    check_order(&alpha)?;
    check_nonnegative(&gamma, "RDP epsilon")?;
    check_probability_strictly_positive(&delta, "delta")?;

    let one = I::<Bk>::point(1.0)?;
    let zero = I::<Bk>::point(0.0)?;

    if alpha.clone().mul(delta.clone())?.upper_f64()? >= 1.0 {
        return Ok(None);
    }

    if gamma.upper_f64()? == 0.0 {
        return Ok(Some(zero));
    }

    let alpha_m1 = alpha.clone().sub(one)?;
    let numerator_arg = alpha_m1.clone().mul(gamma)?;
    if numerator_arg.lower_f64()? <= 0.0 {
        return Ok(None);
    }

    // epsilon = softplus(
    //     log(expm1((alpha - 1) * gamma)) - log(alpha) - log(delta)
    // ) / (alpha - 1)
    let log_ratio = numerator_arg
        .log_expm1()?
        .sub(alpha.ln()?)?
        .sub(delta.ln()?)?;

    Ok(Some(log_ratio.softplus()?.div(alpha_m1)?.max(zero)?))
}

fn check_order<Bk>(alpha: &I<Bk>) -> Fallible<()>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    if alpha.lower_f64()? <= 1.0 {
        return fallible!(
            FailedMap,
            "Renyi order alpha must be strictly greater than one"
        );
    }
    Ok(())
}

fn check_nonnegative<Bk>(value: &I<Bk>, name: &str) -> Fallible<()>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    if value.lower_f64()? < 0.0 {
        return fallible!(FailedMap, "{name} must be non-negative");
    }
    Ok(())
}

fn check_probability_strictly_positive<Bk>(value: &I<Bk>, name: &str) -> Fallible<()>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    if value.lower_f64()? <= 0.0 || value.upper_f64()? > 1.0 {
        return fallible!(FailedMap, "{name} must be in (0, 1]");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::{A, D};

    #[test]
    fn test_fixed_order_kernels_match_oracle() -> Fallible<()> {
        let alpha: f64 = 2.0;
        let gamma: f64 = 0.5;
        let epsilon: f64 = 1.0;
        let expected0 = (alpha - 1.0) * (gamma - epsilon) + alpha * (1.0 - 1.0 / alpha).ln()
            - (alpha - 1.0).ln();
        let actual0 =
            rdp_log_delta0_on::<A>(I::point(alpha)?, I::point(gamma)?, I::point(epsilon)?)?;
        assert!(actual0.lower_f64()? <= expected0);
        assert!(actual0.upper_f64()? >= expected0);

        let expected1 = ((alpha - 1.0) * gamma).exp_m1().ln()
            - alpha.ln()
            - ((alpha - 1.0) * epsilon).exp_m1().ln();
        let actual1 =
            rdp_log_delta1_on::<A>(I::point(alpha)?, I::point(gamma)?, I::point(epsilon)?)?
                .expect("strict epsilon > gamma applicability");
        assert!(actual1.lower_f64()? <= expected1);
        assert!(actual1.upper_f64()? >= expected1);

        let inverse = rdp_epsilon0_on::<D>(I::point(alpha)?, I::point(gamma)?, I::point(1e-3)?)?;
        assert!(inverse.lower_f64()? >= 0.0);
        assert!(inverse.upper_f64()?.is_finite());
        Ok(())
    }

    #[test]
    fn test_theorem_branch_applicability() -> Fallible<()> {
        assert!(rdp_log_delta1_on::<A>(I::point(2.0)?, I::point(0.5)?, I::point(0.5)?,)?.is_none());
        assert!(rdp_epsilon1_on::<A>(I::point(2.0)?, I::point(0.5)?, I::point(0.5)?,)?.is_none());
        assert!(rdp_epsilon0_on::<D>(I::point(1.0)?, I::point(0.5)?, I::point(0.1)?,).is_err());
        Ok(())
    }
}
