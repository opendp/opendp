use crate::{
    error::Fallible,
    measures::curves::{
        LogProfileFn, check_alpha, check_epsilon,
        logspace::{LOG_TRUE_MIN, one_minus_delta_from_log_upper_unchecked},
    },
    traits::{CInterval, SInterval, backend::Dashu},
    utilities::search::{SearchMode, fallible_optimize_to_precision},
};

const EPS_TRUE_MIN: f64 = 744.4400719213812;

type Cert = SInterval<Dashu>; // software interval provides certified transcendental bounds

pub fn beta_via_profile(profile: &LogProfileFn, alpha: f64) -> Fallible<f64> {
    check_alpha(alpha)?;

    if alpha == 0.0 {
        // At alpha=0, both tradeoff branches are bounded by 1-delta(eps),
        // and the first branch attains that bound. Evaluate the profile's
        // actual positive-infinity endpoint instead of assuming it is zero.
        return profile_one_minus_delta(profile, f64::INFINITY)?.lower_f64();
    }
    if alpha == 1.0 {
        return Ok(0.0);
    }

    // Tight conversion:
    //
    // beta(alpha) >= sup_eps max {
    //     1 - delta(eps) - exp(eps) * alpha,
    //     exp(-eps) * max(1 - delta(eps) - alpha, 0)
    // }
    //
    // Fast objectives only choose candidate epsilons. The candidates are then
    // recomputed with DInterval below.
    let beta_t1 = maximize_t1_lower(profile, alpha)?;
    let beta_t2 = maximize_t2_lower(profile, alpha)?;

    Ok(beta_t1.max(beta_t2).clamp(0.0, 1.0))
}

/// Conservative enclosure of `1 - delta(epsilon)`.
///
/// For log-delta profiles, this uses:
///
/// `1 - exp(x) = -expm1(x)`
///
/// which avoids catastrophic cancellation when delta is close to one.
fn profile_one_minus_delta(profile: &LogProfileFn, epsilon: f64) -> Fallible<Cert> {
    check_epsilon(epsilon)?;

    one_minus_delta_from_log_upper_unchecked(profile(epsilon)?)
}

#[inline]
fn positive_part(x: Cert) -> Fallible<Cert> {
    x.max(Cert::point(0.0)?)?.clamp01()
}

fn maximize_t1_lower(profile: &LogProfileFn, alpha: f64) -> Fallible<f64> {
    // For eps >= -ln(alpha), alpha * exp(eps) >= 1,
    // so branch 1 cannot be positive.
    let eps_hi = (-alpha.ln()).next_up().clamp(0.0, EPS_TRUE_MIN);

    let mut best = beta_profile_candidate_t1_lower(profile, alpha, 0.0)?;

    if eps_hi > 0.0 {
        let eps = maximize_unimodal_epsilon(0.0, eps_hi, |eps| {
            beta_profile_candidate_t1_fast(profile, alpha, eps)
        })?;

        best = best.max(beta_profile_candidate_t1_lower(profile, alpha, eps)?);
        best = best.max(beta_profile_candidate_t1_lower(profile, alpha, eps_hi)?);
    }

    Ok(best.clamp(0.0, 1.0))
}

fn maximize_t2_lower(profile: &LogProfileFn, alpha: f64) -> Fallible<f64> {
    // branch 2 <= (1 - alpha) * exp(-eps).
    // Beyond this range, no positive representable f64 lower bound is possible.
    let one_minus_alpha_hi = (CInterval::point(1.0)? - CInterval::point(alpha)?)?
        .clamp01()?
        .upper_f64()?;

    if one_minus_alpha_hi == 0.0 {
        return Ok(0.0);
    }

    let eps_hi = (one_minus_alpha_hi.ln() - LOG_TRUE_MIN)
        .next_up()
        .clamp(0.0, EPS_TRUE_MIN);

    let mut best = beta_profile_candidate_t2_lower(profile, alpha, 0.0)?;

    if eps_hi > 0.0 {
        let eps = maximize_unimodal_epsilon(0.0, eps_hi, |eps| {
            beta_profile_candidate_t2_fast(profile, alpha, eps)
        })?;

        best = best.max(beta_profile_candidate_t2_lower(profile, alpha, eps)?);
        best = best.max(beta_profile_candidate_t2_lower(profile, alpha, eps_hi)?);
    }

    Ok(best.clamp(0.0, 1.0))
}

#[inline]
fn maximize_unimodal_epsilon(
    lo: f64,
    hi: f64,
    objective: impl Fn(f64) -> Fallible<f64>,
) -> Fallible<f64> {
    if !(hi > lo) {
        return Ok(lo);
    }

    let optimum = fallible_optimize_to_precision(SearchMode::Maximize, lo, hi, None, objective)?;

    Ok(optimum.arg.clamp(lo, hi))
}

// Fast, non-certified objective for branch 1:
//
//     t1(eps) = 1 - delta(eps) - alpha * exp(eps).
//
// This only chooses epsilon. Conservativeness comes from the final lower eval.
#[inline]
fn beta_profile_candidate_t1_fast(
    profile: &LogProfileFn,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = one_minus_delta_fast(profile, epsilon)?;

    let z = epsilon + alpha.ln();

    if z >= 0.0 {
        return Ok(0.0);
    }

    Ok((one_minus_delta - z.exp()).clamp(0.0, 1.0))
}

// Fast, non-certified objective for branch 2.
//
// Instead of returning beta-space directly, return log(branch 2):
//
//     log t2(eps) = -eps + log(1 - delta(eps) - alpha).
//
// This has the same maximizer wherever branch 2 is positive.
#[inline]
fn beta_profile_candidate_t2_fast(
    profile: &LogProfileFn,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = one_minus_delta_fast(profile, epsilon)?;
    let base = one_minus_delta - alpha;

    if base <= 0.0 {
        return Ok(f64::NEG_INFINITY);
    }

    Ok(-epsilon + base.ln())
}

#[inline]
fn one_minus_delta_fast(profile: &LogProfileFn, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;

    let value = profile(epsilon)?;

    // Fast path only: stable ordinary-f64 computation of 1 - exp(log_delta).
    Ok((-value.exp_m1()).clamp(0.0, 1.0))
}

// Certified lower evaluation of branch 1:
//
//     1 - delta(eps) - alpha * exp(eps).
#[inline]
fn beta_profile_candidate_t1_lower(
    profile: &LogProfileFn,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = profile_one_minus_delta(profile, epsilon)?;

    // Compute alpha * exp(epsilon) as exp(epsilon + ln(alpha)).
    // This avoids overflowing exp(epsilon) when alpha is tiny.
    let z = (Cert::point(epsilon)? + Cert::point(alpha)?.ln()?)?;

    // If the upper endpoint is nonnegative, then alpha * exp(epsilon) may be
    // at least one, and returning zero is a conservative lower bound.
    if z.upper_f64()? >= 0.0 {
        return Ok(0.0);
    }

    let alpha_exp_eps = z.exp()?;

    positive_part((one_minus_delta - alpha_exp_eps)?)?.lower_f64()
}

// Certified lower evaluation of branch 2:
//
//     exp(-eps) * max(1 - delta(eps) - alpha, 0).
#[inline]
fn beta_profile_candidate_t2_lower(
    profile: &LogProfileFn,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = profile_one_minus_delta(profile, epsilon)?;

    let base = positive_part((one_minus_delta - Cert::point(alpha)?)?)?;

    if base.upper_f64()? == 0.0 {
        return Ok(0.0);
    }

    ((Cert::point(-epsilon)?.exp()?) * base)?
        .clamp01()?
        .lower_f64()
}
