//! Fixed-order RDP to approximate-DP conversion kernels.
//!
//! These formulas are generic over the interval scalar:
//!
//! - instantiate with `N64<Approximate>` while selecting a promising Renyi order;
//! - instantiate with `SoftFloat` for the final certified evaluation.
//!
//! The search instantiation affects only tightness. The certified instantiation
//! is the privacy-critical evaluation.
//!
//! The implementation builds these elementary functions from the interval
//! arithmetic and transcendental operators.

use crate::{
    error::Fallible,
    traits::{DirectedScalar, DirectedTranscendental, Interval},
};

type I<S> = Interval<S>;

fn ln_1p<S: DirectedTranscendental>(value: I<S>) -> Fallible<I<S>> {
    (I::<S>::point(1.0)? + value)?.ln()
}

fn log_expm1<S: DirectedTranscendental>(value: I<S>) -> Fallible<I<S>> {
    if value.lower_f64()? >= std::f64::consts::LN_2 {
        // log(expm1(x)) = x + log1p(-exp(-x)). This avoids overflowing
        // exp(x) for the large arguments induced by very high Renyi orders.
        let exp_neg = (-value.clone()).exp()?;
        let correction = ln_1p(-exp_neg)?;
        value + correction
    } else {
        // expm1 avoids cancellation as the Renyi order approaches one.
        value.exp_m1()?.ln()
    }
}

fn softplus<S: DirectedTranscendental>(value: I<S>) -> Fallible<I<S>> {
    if value.lower_f64()? >= 0.0 {
        // softplus(x) = x + log1p(exp(-x)) is stable for large positive x.
        let correction = ln_1p((-value.clone()).exp()?)?;
        value + correction
    } else {
        ln_1p(value.exp()?)
    }
}

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
pub(crate) fn rdp_log_delta0_on<S: DirectedTranscendental>(
    alpha: I<S>,
    gamma: I<S>,
    epsilon: I<S>,
) -> Fallible<I<S>> {
    check_order(&alpha)?;
    check_nonnegative(&gamma, "RDP epsilon")?;
    check_nonnegative(&epsilon, "epsilon")?;

    let one = I::<S>::point(1.0)?;
    let alpha_m1 = (alpha.clone() - one.clone())?;

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
    let neg_recip_alpha = -(one / alpha.clone())?;
    let correction = ((alpha.clone() * ln_1p(neg_recip_alpha)?)? - alpha_m1.clone().ln()?)?;

    (alpha_m1 * (gamma - epsilon)?)? + correction
}

/// Fixed-order Asoodeh et al. upper bound on `log(delta)`.
///
/// Returns `None` when the theorem's strict `epsilon > gamma` condition cannot
/// be established with the selected interval backend.
pub(crate) fn rdp_log_delta1_on<S: DirectedTranscendental>(
    alpha: I<S>,
    gamma: I<S>,
    epsilon: I<S>,
) -> Fallible<Option<I<S>>> {
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

    let one = I::<S>::point(1.0)?;
    let alpha_m1 = (alpha.clone() - one)?;
    let numerator_arg = (alpha_m1.clone() * gamma)?;
    let denominator_arg = (alpha_m1 * epsilon)?;

    if numerator_arg.lower_f64()? <= 0.0 || denominator_arg.lower_f64()? <= 0.0 {
        return Ok(None);
    }

    let value = ((log_expm1(numerator_arg)? - alpha.ln()?)? - log_expm1(denominator_arg)?)?;

    Ok(Some(value))
}

/// Fixed-order Balle--Canonne upper bound on epsilon at a requested delta.
///
/// This is the exact algebraic inverse of [`rdp_log_delta0_on`] at fixed
/// `(alpha, gamma)`, evaluated with the selected interval backend.
pub(crate) fn rdp_epsilon0_on<S: DirectedTranscendental>(
    alpha: I<S>,
    gamma: I<S>,
    delta: I<S>,
) -> Fallible<I<S>> {
    check_order(&alpha)?;
    check_nonnegative(&gamma, "RDP epsilon")?;
    check_probability_strictly_positive(&delta, "delta")?;

    let one = I::<S>::point(1.0)?;
    let zero = I::<S>::point(0.0)?;
    let alpha_m1 = (alpha.clone() - one.clone())?;

    let neg_recip_alpha = -(one / alpha.clone())?;
    let correction = ((alpha.clone() * ln_1p(neg_recip_alpha)?)? - alpha_m1.clone().ln()?)?;

    let correction_delta = (correction - delta.ln()?)?;
    let correction_ratio = (correction_delta / alpha_m1)?;
    Ok((gamma + correction_ratio)?.max(zero))
}

/// Fixed-order Asoodeh et al. upper bound on epsilon at a requested delta.
///
/// Returns `None` when the theorem's strict `alpha * delta < 1` condition
/// cannot be established with the selected interval backend.
pub(crate) fn rdp_epsilon1_on<S: DirectedTranscendental>(
    alpha: I<S>,
    gamma: I<S>,
    delta: I<S>,
) -> Fallible<Option<I<S>>> {
    check_order(&alpha)?;
    check_nonnegative(&gamma, "RDP epsilon")?;
    check_probability_strictly_positive(&delta, "delta")?;

    let one = I::<S>::point(1.0)?;
    let zero = I::<S>::point(0.0)?;

    if (alpha.clone() * delta.clone())?.upper_f64()? >= 1.0 {
        return Ok(None);
    }

    if gamma.upper_f64()? == 0.0 {
        return Ok(Some(zero));
    }

    let alpha_m1 = (alpha.clone() - one)?;
    let numerator_arg = (alpha_m1.clone() * gamma)?;
    if numerator_arg.lower_f64()? <= 0.0 {
        return Ok(None);
    }

    // epsilon = softplus(
    //     log(expm1((alpha - 1) * gamma)) - log(alpha) - log(delta)
    // ) / (alpha - 1)
    let log_ratio = ((log_expm1(numerator_arg)? - alpha.ln()?)? - delta.ln()?)?;

    Ok(Some((softplus(log_ratio)? / alpha_m1)?.max(zero)))
}

fn check_order<S: DirectedScalar>(alpha: &I<S>) -> Fallible<()> {
    if alpha.lower_f64()? <= 1.0 {
        return fallible!(
            FailedMap,
            "Renyi order alpha must be strictly greater than one"
        );
    }
    Ok(())
}

fn check_nonnegative<S: DirectedScalar>(value: &I<S>, name: &str) -> Fallible<()> {
    if value.lower_f64()? < 0.0 {
        return fallible!(FailedMap, "{name} must be non-negative");
    }
    Ok(())
}

fn check_probability_strictly_positive<S: DirectedScalar>(
    value: &I<S>,
    name: &str,
) -> Fallible<()> {
    if value.lower_f64()? <= 0.0 || value.upper_f64()? > 1.0 {
        return fallible!(FailedMap, "{name} must be in (0, 1]");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::{Approximate, N64, SoftFloat};

    #[test]
    fn test_fixed_order_kernels_match_oracle() -> Fallible<()> {
        let alpha: f64 = 2.0;
        let gamma: f64 = 0.5;
        let epsilon: f64 = 1.0;
        let expected0 = (alpha - 1.0) * (gamma - epsilon) + alpha * (1.0 - 1.0 / alpha).ln()
            - (alpha - 1.0).ln();
        let actual0 = rdp_log_delta0_on::<N64<Approximate>>(
            I::point(alpha)?,
            I::point(gamma)?,
            I::point(epsilon)?,
        )?;
        assert!(actual0.lower_f64()? <= expected0);
        assert!(actual0.upper_f64()? >= expected0);

        let expected1 = ((alpha - 1.0) * gamma).exp_m1().ln()
            - alpha.ln()
            - ((alpha - 1.0) * epsilon).exp_m1().ln();
        let actual1 = rdp_log_delta1_on::<N64<Approximate>>(
            I::point(alpha)?,
            I::point(gamma)?,
            I::point(epsilon)?,
        )?
        .expect("strict epsilon > gamma applicability");
        assert!(actual1.lower_f64()? <= expected1);
        assert!(actual1.upper_f64()? >= expected1);

        // Final privacy-curve evaluation uses the software interval backend.
        let certified0 =
            rdp_log_delta0_on::<SoftFloat>(I::point(alpha)?, I::point(gamma)?, I::point(epsilon)?)?;
        assert!(certified0.lower_f64()? <= expected0);
        assert!(certified0.upper_f64()? >= expected0);

        let certified1 =
            rdp_log_delta1_on::<SoftFloat>(I::point(alpha)?, I::point(gamma)?, I::point(epsilon)?)?
                .expect("strict epsilon > gamma applicability");
        assert!(certified1.lower_f64()? <= expected1);
        assert!(certified1.upper_f64()? >= expected1);

        let inverse =
            rdp_epsilon0_on::<SoftFloat>(I::point(alpha)?, I::point(gamma)?, I::point(1e-3)?)?;
        assert!(inverse.lower_f64()? >= 0.0);
        assert!(inverse.upper_f64()?.is_finite());
        Ok(())
    }

    #[test]
    fn test_fixed_order_kernels_near_one_and_large_orders() -> Fallible<()> {
        let cases = [
            (1.0000000149011612, 0.5, 1.0),
            (2f64.powi(53), 0.5, 1.0),
            (1e150, 1e-150, 2e-150),
        ];

        use rug::Float;
        let log_expm1_oracle = |x: Float| {
            if x >= std::f64::consts::LN_2 {
                let correction = (-(-x.clone()).exp()).ln_1p();
                x + correction
            } else {
                x.exp_m1().ln()
            }
        };

        for (alpha, gamma, epsilon) in cases {
            // The certified interval may be much tighter than ordinary-f64
            // cancellation in this formula near alpha=1. Use an independent
            // high-precision oracle instead of requiring it to enclose that error.
            let a = Float::with_val(512, alpha);
            let a_m1: Float = a.clone() - 1.0_f64;
            let g = Float::with_val(512, gamma);
            let e = Float::with_val(512, epsilon);
            let expected0 = a_m1.clone() * (g.clone() - e.clone())
                + a.clone() * (-Float::with_val(512, 1) / a.clone()).ln_1p()
                - a_m1.clone().ln();
            let actual0 = rdp_log_delta0_on::<SoftFloat>(
                I::point(alpha)?,
                I::point(gamma)?,
                I::point(epsilon)?,
            )?;
            assert!(Float::with_val(512, actual0.lower_f64()?) <= expected0);
            assert!(Float::with_val(512, actual0.upper_f64()?) >= expected0);

            let expected1 =
                log_expm1_oracle(a_m1.clone() * g) - a.ln() - log_expm1_oracle(a_m1 * e);
            let actual1 = rdp_log_delta1_on::<SoftFloat>(
                I::point(alpha)?,
                I::point(gamma)?,
                I::point(epsilon)?,
            )?
            .expect("strict epsilon > gamma applicability");
            assert!(Float::with_val(512, actual1.lower_f64()?) <= expected1);
            assert!(Float::with_val(512, actual1.upper_f64()?) >= expected1);
        }
        Ok(())
    }

    #[test]
    fn test_theorem_branch_applicability() -> Fallible<()> {
        assert!(
            rdp_log_delta1_on::<N64<Approximate>>(I::point(2.0)?, I::point(0.5)?, I::point(0.5)?,)?
                .is_none()
        );
        assert!(
            rdp_epsilon1_on::<N64<Approximate>>(I::point(2.0)?, I::point(0.5)?, I::point(0.5)?,)?
                .is_none()
        );
        assert!(
            rdp_epsilon0_on::<SoftFloat>(I::point(1.0)?, I::point(0.5)?, I::point(0.1)?,).is_err()
        );
        Ok(())
    }
}
