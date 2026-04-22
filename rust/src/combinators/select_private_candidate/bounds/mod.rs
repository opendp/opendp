use crate::{
    core::Function,
    error::Fallible,
    traits::{InfAdd, InfDiv, InfExp, InfExpM1, InfLn1P, InfMul, InfSub},
};

#[cfg(test)]
mod test;

fn ensure_open_unit_interval(x: f64) -> Fallible<()> {
    if x <= 0.0 || x >= 1.0 {
        return fallible!(
            MakeMeasurement,
            "failed to compute the repetition distribution parameters"
        );
    }
    Ok(())
}

/// Solve for the internal exponential parameter `x` from the requested NB/logarithmic mean.
///
/// # Proof Definition
/// For any valid `mean > 1` and `eta >= 0`,
/// either returns `Err(e)` if the search fails,
/// or returns `Ok(x)` where `x` is the final upper endpoint of the bisection search
/// and satisfies `expected_nb_mean_from_x_hi(eta, x) <= mean`.
///
/// # Paper Reference
/// Papernot–Steinke (2021), Definition 1.
///
/// # Note
/// This implementation uses the conservatively rounded upper evaluator
/// `expected_nb_mean_from_x_hi` inside the bisection. The caller should round the final
/// `x` upward before using it in the sampler.
pub fn solve_nb_x_from_mean(mean: f64, eta: f64) -> Fallible<f64> {
    let mut lo = 0.0;
    let mut hi = 1.0;

    while expected_nb_mean_from_x_hi(eta, hi)? > mean {
        hi *= 2.0;
        if !hi.is_finite() {
            return fallible!(
                MakeMeasurement,
                "failed to solve the repetition distribution parameter from mean"
            );
        }
    }

    loop {
        let mid = lo + (hi - lo) / 2.0;

        // Converged: no representable float remains between lo and hi.
        if mid == lo || mid == hi {
            return Ok(hi);
        }

        if expected_nb_mean_from_x_hi(eta, mid)? > mean {
            lo = mid;
        } else {
            hi = mid;
        }
    }
}

/// Conservative upper bound on the NB/logarithmic expected repetition count at internal
/// parameter `x`.
///
/// For `eta = 0`, this upper-bounds
/// `exp(-x) / (gamma * log(1 / gamma))`.
///
/// For `eta > 0`, this upper-bounds
/// `eta * exp(-x) / (gamma * (1 - gamma^eta))`.
///
/// Here `gamma = 1 - exp(-x)`.
///
/// # Proof Definition
/// For any valid `eta >= 0` and `x > 0`,
/// either returns `Err(e)` if the expression is numerically invalid,
/// or `Ok(out)` where `out` is an upper enclosure of the Definition 1 expectation.
///
/// # Paper Reference
/// Papernot–Steinke (2021), Definition 1.
fn expected_nb_mean_from_x_hi(eta: f64, x: f64) -> Fallible<f64> {
    let q_hi = (-x).inf_exp()?;
    let gamma_lo = gamma_lo_from_x(x)?;
    ensure_open_unit_interval(gamma_lo)?;

    if eta == 0.0 {
        let log_inv_gamma_lo = log_inv_gamma_lo_from_x(x)?;
        let denom_lo = gamma_lo.neg_inf_mul(&log_inv_gamma_lo)?;
        if denom_lo <= 0.0 {
            return Ok(f64::INFINITY);
        }
        return q_hi.inf_div(&denom_lo);
    }

    let gamma_eta_hi = gamma_pow_eta_hi_from_x(eta, x)?;
    let one_minus_gamma_eta_lo = 1.0f64.neg_inf_sub(&gamma_eta_hi)?;
    if one_minus_gamma_eta_lo <= 0.0 {
        return Ok(f64::INFINITY);
    }

    let denom_lo = gamma_lo.neg_inf_mul(&one_minus_gamma_eta_lo)?;
    if denom_lo <= 0.0 {
        return Ok(f64::INFINITY);
    }

    let num_hi = eta.inf_mul(&q_hi)?;
    num_hi.inf_div(&denom_lo)
}

/// Conservative lower bound on `gamma = 1 - exp(-x)`.
///
/// # Proof Definition
/// For any finite `x`,
/// either returns `Err(e)` if the enclosure fails,
/// or returns `Ok(out)` where `out <= 1 - exp(-x)`.
fn gamma_lo_from_x(x: f64) -> Fallible<f64> {
    // gamma = -(exp_m1(-x))
    let expm1_hi = (-x).inf_exp_m1()?;
    Ok(-expm1_hi)
}

/// Conservative upper bound on `gamma = 1 - exp(-x)`.
///
/// # Proof Definition
/// For any finite `x`,
/// either returns `Err(e)` if the enclosure fails,
/// or returns `Ok(out)` where `out >= 1 - exp(-x)`.
fn gamma_hi_from_x(x: f64) -> Fallible<f64> {
    let expm1_lo = (-x).neg_inf_exp_m1()?;
    Ok(-expm1_lo)
}

/// Conservative lower bound on `log(1 / gamma)`, where `gamma = 1 - exp(-x)`.
///
/// # Proof Definition
/// For any finite `x`,
/// either returns `Err(e)` if the enclosure fails,
/// or returns `Ok(out)` where `out <= log(1 / (1 - exp(-x)))`.
fn log_inv_gamma_lo_from_x(x: f64) -> Fallible<f64> {
    let gamma_hi = gamma_hi_from_x(x)?;
    ensure_open_unit_interval(gamma_hi)?;

    // log(1 / gamma) = -ln(gamma) = -ln1p(gamma - 1)
    let gamma_minus_one_hi = gamma_hi.inf_sub(&1.0)?;
    let ln_gamma_hi = gamma_minus_one_hi.inf_ln_1p()?;
    Ok(-ln_gamma_hi)
}

/// Conservative upper bound on `log(1 / gamma)`, where `gamma = 1 - exp(-x)`.
///
/// # Proof Definition
/// For any finite `x`,
/// either returns `Err(e)` if the enclosure fails,
/// or returns `Ok(out)` where `out >= log(1 / (1 - exp(-x)))`.
fn log_inv_gamma_hi_from_x(x: f64) -> Fallible<f64> {
    let gamma_lo = gamma_lo_from_x(x)?;
    ensure_open_unit_interval(gamma_lo)?;

    let gamma_minus_one_lo = gamma_lo.neg_inf_sub(&1.0)?;
    let ln_gamma_lo = gamma_minus_one_lo.neg_inf_ln_1p()?;
    Ok(-ln_gamma_lo)
}

/// Conservative upper bound on `gamma^eta`, where `gamma = 1 - exp(-x)`.
///
/// # Proof Definition
/// For any valid `eta >= 0` and finite `x`,
/// either returns `Err(e)` if the enclosure fails,
/// or returns `Ok(out)` where `out >= (1 - exp(-x))^eta`.
fn gamma_pow_eta_hi_from_x(eta: f64, x: f64) -> Fallible<f64> {
    let gamma_hi = gamma_hi_from_x(x)?;
    ensure_open_unit_interval(gamma_hi)?;

    let gamma_minus_one_hi = gamma_hi.inf_sub(&1.0)?;
    let ln_gamma_hi = gamma_minus_one_hi.inf_ln_1p()?;
    let exponent_hi = eta.inf_mul(&ln_gamma_hi)?;
    exponent_hi.inf_exp()
}

/// Conservative lower bound on `log(alpha / (alpha - 1))`.
///
/// This is the admissibility threshold in Theorem 6:
/// `exp(eps_hat) <= 1 + 1 / (alpha - 1)`,
/// equivalently
/// `eps_hat <= log(alpha / (alpha - 1))`.
///
/// # Proof Definition
/// For any `alpha > 1`,
/// either returns `Err(e)` if the enclosure fails,
/// or returns `Ok(out)` where `out <= log(alpha / (alpha - 1))`.
fn log_alpha_over_alpha_minus_one_lo(alpha: f64) -> Fallible<f64> {
    // log(alpha / (alpha - 1)) = -ln(1 - 1 / alpha)
    let inv_alpha_lo = 1.0f64.neg_inf_div(&alpha)?;
    let neg_inv_alpha_hi = -inv_alpha_lo;
    let ln_one_minus_inv_alpha_hi = neg_inv_alpha_hi.inf_ln_1p()?;
    Ok(-ln_one_minus_inv_alpha_hi)
}

/// Conservative upper bound on `log(mean)`, for `mean > 0`.
///
/// # Proof Definition
/// For any positive finite `mean`,
/// either returns `Err(e)` if the enclosure fails,
/// or returns `Ok(out)` where `out >= log(mean)`.
fn log_mean_hi(mean: f64) -> Fallible<f64> {
    if !(mean > 0.0 && mean.is_finite()) {
        return fallible!(MakeMeasurement, "mean must be positive and finite");
    }

    let mean_minus_one_hi = mean.inf_sub(&1.0)?;
    mean_minus_one_hi.inf_ln_1p()
}

/// Construct the Rényi-DP curve for thresholded geometric / conditional sampling.
///
/// This implements the second inequality of Corollary 16:
/// `D_alpha(Q_S || Q'_S)`
/// `<= D_alpha(Q || Q')`
/// ` + ((alpha - 2) / (alpha - 1)) D_{alpha - 1}(Q' || Q)`
/// ` + (2 / (alpha - 1)) log(1 / Q(S))`
///
/// specialized to the thresholded geometric case, where `Q(S) = gamma = 1 - exp(-x)`.
///
/// # Proof Definition
/// For any base Rényi-DP upper bound `base_curve` and valid `x`,
/// returns a function that upper-bounds the thresholded / conditional-sampling
/// Rényi-DP privacy curve.
///
/// # Paper Reference
/// Papernot–Steinke (2021), Corollary 16 (second inequality).
pub fn new_conditional_rdp_curve(base_curve: Function<f64, f64>, x: f64) -> Function<f64, f64> {
    Function::new_fallible(move |alpha: &f64| {
        if *alpha <= 2.0 {
            return Ok(f64::INFINITY);
        }

        let eps_alpha = base_curve.eval(alpha)?;

        // By monotonicity of Rényi divergence in the order,
        // evaluating at an upward-rounded (alpha - 1) is conservative.
        let alpha_minus_one_hi = alpha.inf_sub(&1.0)?;
        let eps_alpha_minus_one_hi = base_curve.eval(&alpha_minus_one_hi)?;

        let alpha_minus_two_hi = alpha.inf_sub(&2.0)?;
        let alpha_minus_one_lo = alpha.neg_inf_sub(&1.0)?;
        let coeff_hi = alpha_minus_two_hi.inf_div(&alpha_minus_one_lo)?;

        let log_inv_gamma_hi = log_inv_gamma_hi_from_x(x)?;
        let term1_hi = coeff_hi.inf_mul(&eps_alpha_minus_one_hi)?;

        let term2_num_hi = 2.0f64.inf_mul(&log_inv_gamma_hi)?;
        let term2_hi = term2_num_hi.inf_div(&alpha_minus_one_lo)?;

        eps_alpha.inf_add(&term1_hi)?.inf_add(&term2_hi)
    })
}

/// Construct the Rényi-DP curve for truncated-negative-binomial best-of-k.
///
/// This implements Theorem 2, specialized with the base curve used for both Rényi orders.
/// The requested `mean` is used conservatively in the `log E[K] / (alpha - 1)` term.
///
/// # Proof Definition
/// For any base Rényi-DP upper bound `base_curve`, valid `eta`, valid `x`, and valid `mean`,
/// returns a function implementing an upper bound on the Theorem 2 privacy curve.
///
/// # Paper Reference
/// Papernot–Steinke (2021), Theorem 2.
pub fn new_negative_binomial_rdp_curve(
    base_curve: Function<f64, f64>,
    eta: f64,
    x: f64,
    mean: f64,
) -> Function<f64, f64> {
    Function::new_fallible(move |alpha: &f64| {
        if *alpha <= 1.0 {
            return Ok(f64::INFINITY);
        }

        let eps_alpha = base_curve.eval(alpha)?;

        let one_plus_eta_hi = 1.0f64.inf_add(&eta)?;
        let alpha_minus_one_lo = alpha.neg_inf_sub(&1.0)?;

        let inv_alpha_lo = 1.0f64.neg_inf_div(alpha)?;
        let one_minus_inv_alpha_hi = 1.0f64.inf_sub(&inv_alpha_lo)?;

        let coeff1_hi = one_plus_eta_hi.inf_mul(&one_minus_inv_alpha_hi)?;
        let term1_hi = coeff1_hi.inf_mul(&eps_alpha)?;

        let log_inv_gamma_hi = log_inv_gamma_hi_from_x(x)?;
        let term2_num_hi = one_plus_eta_hi.inf_mul(&log_inv_gamma_hi)?;
        let term2_hi = term2_num_hi.inf_div(alpha)?;

        let log_mean_hi = log_mean_hi(mean)?;
        let term3_hi = log_mean_hi.inf_div(&alpha_minus_one_lo)?;

        eps_alpha
            .inf_add(&term1_hi)?
            .inf_add(&term2_hi)?
            .inf_add(&term3_hi)
    })
}

/// Construct the Rényi-DP curve for Poisson best-of-k.
///
/// This implements Theorem 6 together with the standard conversion from
/// `(alpha, eps(alpha))`-RDP to an approximate-DP witness
/// `(eps(alpha), delta(alpha))`, where
/// `delta(alpha) = alpha^{-1} (1 - alpha^{-1})^{alpha - 1}`.
///
/// The admissibility condition in Theorem 6 is
/// `exp(eps_hat) <= 1 + 1 / (alpha - 1)`,
/// equivalently
/// `eps_hat <= log(alpha / (alpha - 1))`; this function checks that
/// condition conservatively.
///
/// # Proof Definition
/// For any base Rényi-DP upper bound `base_curve` and valid `mean`,
/// returns a function implementing an upper bound on the Poisson privacy curve.
///
/// # Paper Reference
/// Papernot–Steinke (2021), Theorem 6, combined with the standard
/// RDP-to-approximate-DP conversion.
pub fn new_poisson_rdp_curve(base_curve: Function<f64, f64>, mean: f64) -> Function<f64, f64> {
    Function::new_fallible(move |alpha: &f64| {
        if mean == 0.0 {
            return Ok(0.0);
        }
        if *alpha <= 1.0 {
            return Ok(f64::INFINITY);
        }

        let eps_alpha = base_curve.eval(alpha)?;

        let max_eps_lo = log_alpha_over_alpha_minus_one_lo(*alpha)?;
        if eps_alpha > max_eps_lo {
            return Ok(f64::INFINITY);
        }

        // delta(alpha) = alpha^{-1} (1 - alpha^{-1})^{alpha - 1}
        let inv_alpha_hi = 1.0f64.inf_div(alpha)?;
        let inv_alpha_lo = 1.0f64.neg_inf_div(alpha)?;

        let one_minus_inv_alpha_hi = 1.0f64.inf_sub(&inv_alpha_lo)?;
        let one_minus_inv_alpha_minus_one_hi = one_minus_inv_alpha_hi.inf_sub(&1.0)?;
        let ln_one_minus_inv_alpha_hi = one_minus_inv_alpha_minus_one_hi.inf_ln_1p()?;

        // Since ln(1 - 1/alpha) < 0, using a lower bound on (alpha - 1)
        // and an upper bound on ln(1 - 1/alpha) gives an upper bound on the exponent.
        let alpha_minus_one_lo = alpha.neg_inf_sub(&1.0)?;
        let exponent_hi = alpha_minus_one_lo.inf_mul(&ln_one_minus_inv_alpha_hi)?;
        let power_hi = exponent_hi.inf_exp()?;

        let delta_hi = inv_alpha_hi.inf_mul(&power_hi)?;

        let term1_hi = mean.inf_mul(&delta_hi)?;
        let log_mean_hi = log_mean_hi(mean)?;
        let term2_hi = log_mean_hi.inf_div(&alpha_minus_one_lo)?;

        eps_alpha.inf_add(&term1_hi)?.inf_add(&term2_hi)
    })
}
