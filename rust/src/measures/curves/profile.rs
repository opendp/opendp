use crate::{
    error::Fallible,
    measures::curves::{ProfileFn, ProfileScale, check_alpha, check_delta, check_epsilon},
    traits::{CInterval, DInterval},
    utilities::search::{SearchMode, optimize_to_precision},
};

const F64_TRUE_MIN: f64 = f64::from_bits(1);
const LOG_TRUE_MIN: f64 = -744.4400719213812;
const EPS_TRUE_MIN: f64 = 744.4400719213812;

// Finite sentinel for log-objectives where the actual value is -infinity.
// Avoids depending on search helpers accepting non-finite objective values.
const FAST_NEG_INF: f64 = -1.0e300;

type Cert = DInterval;

pub fn delta_via_profile(profile: &ProfileFn, scale: ProfileScale, epsilon: f64) -> Fallible<f64> {
    profile_delta(profile, scale, epsilon)?.upper_f64()
}

pub fn beta_via_profile(profile: &ProfileFn, scale: ProfileScale, alpha: f64) -> Fallible<f64> {
    check_alpha(alpha)?;

    if alpha == 0.0 {
        return Ok(1.0);
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
    let beta_t1 = maximize_t1_lower(profile, scale, alpha)?;
    let beta_t2 = maximize_t2_lower(profile, scale, alpha)?;

    Ok(beta_t1.max(beta_t2).clamp(0.0, 1.0))
}

/// Conservative enclosure of `delta(epsilon)`.
///
/// The profile contracts used here are one-sided: a delta profile returns a
/// conservative upper value, and a log-delta profile returns a conservative
/// upper log-delta. For the beta lower-bound formulas, the only information we
/// need from this enclosure is its upper endpoint, via `1 - delta`.
fn profile_delta(profile: &ProfileFn, scale: ProfileScale, epsilon: f64) -> Fallible<Cert> {
    check_epsilon(epsilon)?;

    let value = profile(epsilon)?;

    match scale {
        ProfileScale::Delta => {
            check_delta(value)?;
            Cert::between(0.0, value)
        }

        ProfileScale::LogDelta => {
            if value.is_nan() || value > 0.0 {
                return fallible!(FailedMap, "log-profile returned invalid log(delta)");
            }

            if value == f64::NEG_INFINITY {
                return Cert::point(0.0);
            }

            // Widen upward because delta = exp(log_delta), and we need an
            // upper bound on delta.
            let log_delta_hi = value.next_up().min(0.0);

            let delta_hi = if log_delta_hi < LOG_TRUE_MIN {
                F64_TRUE_MIN
            } else {
                Cert::point(log_delta_hi)?
                    .exp()?
                    .upper_f64()?
                    .clamp(0.0, 1.0)
            };

            Cert::between(0.0, delta_hi)
        }
    }
}

#[inline]
fn one_minus_delta(delta: Cert) -> Fallible<Cert> {
    Cert::point(1.0)?.sub(delta)?.clamp01()
}

/// Conservative enclosure of `1 - delta(epsilon)`.
///
/// For log-delta profiles, this uses:
///
///     1 - exp(x) = -expm1(x)
///
/// which avoids catastrophic cancellation when delta is close to one.
fn profile_one_minus_delta(
    profile: &ProfileFn,
    scale: ProfileScale,
    epsilon: f64,
) -> Fallible<Cert> {
    check_epsilon(epsilon)?;

    let value = profile(epsilon)?;

    match scale {
        ProfileScale::Delta => {
            check_delta(value)?;
            one_minus_delta(Cert::between(0.0, value)?)
        }

        ProfileScale::LogDelta => {
            if value.is_nan() || value > 0.0 {
                return fallible!(FailedMap, "log-profile returned invalid log(delta)");
            }

            if value == f64::NEG_INFINITY {
                return Cert::point(1.0);
            }

            // Widen upward because this value is used as an upper bound on
            // log(delta), which gives a lower bound on 1 - delta.
            let log_delta_hi = value.next_up().min(0.0);

            if log_delta_hi < LOG_TRUE_MIN {
                return one_minus_delta(Cert::between(0.0, F64_TRUE_MIN)?);
            }

            // expm1 is increasing. Therefore exp_m1([log_delta_hi, log_delta_hi])
            // contains an upper endpoint for expm1(log_delta). Subtracting this
            // interval from zero gives a conservative lower endpoint for
            // 1 - exp(log_delta).
            Cert::point(0.0)?
                .sub(Cert::point(log_delta_hi)?.exp_m1()?)?
                .clamp01()
        }
    }
}

#[inline]
fn positive_part(x: Cert) -> Fallible<Cert> {
    x.max(Cert::point(0.0)?)?.clamp01()
}

fn maximize_t1_lower(profile: &ProfileFn, scale: ProfileScale, alpha: f64) -> Fallible<f64> {
    // For eps >= -ln(alpha), alpha * exp(eps) >= 1,
    // so branch 1 cannot be positive.
    let eps_hi = (-alpha.ln()).next_up().clamp(0.0, EPS_TRUE_MIN);

    let mut best = beta_profile_candidate_t1_lower(profile, scale, alpha, 0.0)?;

    if eps_hi > 0.0 {
        let eps = maximize_unimodal_epsilon(0.0, eps_hi, |eps| {
            beta_profile_candidate_t1_fast(profile, scale, alpha, eps)
        });

        best = best.max(beta_profile_candidate_t1_lower(profile, scale, alpha, eps)?);
        best = best.max(beta_profile_candidate_t1_lower(
            profile, scale, alpha, eps_hi,
        )?);
    }

    Ok(best.clamp(0.0, 1.0))
}

fn maximize_t2_lower(profile: &ProfileFn, scale: ProfileScale, alpha: f64) -> Fallible<f64> {
    // branch 2 <= (1 - alpha) * exp(-eps).
    // Beyond this range, no positive representable f64 lower bound is possible.
    let one_minus_alpha_hi = CInterval::point(1.0)?
        .sub(CInterval::point(alpha)?)?
        .clamp01()?
        .upper_f64()?;

    if one_minus_alpha_hi == 0.0 {
        return Ok(0.0);
    }

    let eps_hi = (one_minus_alpha_hi.ln() - LOG_TRUE_MIN)
        .next_up()
        .clamp(0.0, EPS_TRUE_MIN);

    let mut best = beta_profile_candidate_t2_lower(profile, scale, alpha, 0.0)?;

    if eps_hi > 0.0 {
        let eps = maximize_unimodal_epsilon(0.0, eps_hi, |eps| {
            beta_profile_candidate_t2_fast(profile, scale, alpha, eps)
        });

        best = best.max(beta_profile_candidate_t2_lower(profile, scale, alpha, eps)?);
        best = best.max(beta_profile_candidate_t2_lower(
            profile, scale, alpha, eps_hi,
        )?);
    }

    Ok(best.clamp(0.0, 1.0))
}

#[inline]
fn maximize_unimodal_epsilon(lo: f64, hi: f64, objective: impl Fn(f64) -> Fallible<f64>) -> f64 {
    if !(hi > lo) {
        return lo;
    }

    optimize_to_precision(
        SearchMode::Maximize,
        lo,
        hi,
        None,
        |epsilon| match objective(epsilon) {
            Ok(value) => value,
            Err(_) => SearchMode::Maximize.bad_value(),
        },
    )
    .arg
    .clamp(lo, hi)
}

// Fast, non-certified objective for branch 1:
//
//     t1(eps) = 1 - delta(eps) - alpha * exp(eps).
//
// This only chooses epsilon. Conservativeness comes from the final lower eval.
#[inline]
fn beta_profile_candidate_t1_fast(
    profile: &ProfileFn,
    scale: ProfileScale,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = one_minus_delta_fast(profile, scale, epsilon)?;

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
    profile: &ProfileFn,
    scale: ProfileScale,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = one_minus_delta_fast(profile, scale, epsilon)?;
    let base = one_minus_delta - alpha;

    if base <= 0.0 {
        return Ok(FAST_NEG_INF);
    }

    Ok(-epsilon + base.ln())
}

#[inline]
fn one_minus_delta_fast(profile: &ProfileFn, scale: ProfileScale, epsilon: f64) -> Fallible<f64> {
    check_epsilon(epsilon)?;

    let value = profile(epsilon)?;

    match scale {
        ProfileScale::Delta => {
            check_delta(value)?;
            Ok((1.0 - value).clamp(0.0, 1.0))
        }

        ProfileScale::LogDelta => {
            if value.is_nan() || value > 0.0 {
                return fallible!(FailedMap, "log-profile returned invalid log(delta)");
            }

            if value == f64::NEG_INFINITY {
                return Ok(1.0);
            }

            // Fast path only: stable ordinary-f64 computation of 1 - exp(log_delta).
            Ok((-value.exp_m1()).clamp(0.0, 1.0))
        }
    }
}

// Certified lower evaluation of branch 1:
//
//     1 - delta(eps) - alpha * exp(eps).
#[inline]
fn beta_profile_candidate_t1_lower(
    profile: &ProfileFn,
    scale: ProfileScale,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = profile_one_minus_delta(profile, scale, epsilon)?;

    // Compute alpha * exp(epsilon) as exp(epsilon + ln(alpha)).
    // This avoids overflowing exp(epsilon) when alpha is tiny.
    let z = Cert::point(epsilon)?.add(Cert::point(alpha)?.ln()?)?;

    // If the upper endpoint is nonnegative, then alpha * exp(epsilon) may be
    // at least one, and returning zero is a conservative lower bound.
    if z.upper_f64()? >= 0.0 {
        return Ok(0.0);
    }

    let alpha_exp_eps = z.exp()?;

    positive_part(one_minus_delta.sub(alpha_exp_eps)?)?.lower_f64()
}

// Certified lower evaluation of branch 2:
//
//     exp(-eps) * max(1 - delta(eps) - alpha, 0).
#[inline]
fn beta_profile_candidate_t2_lower(
    profile: &ProfileFn,
    scale: ProfileScale,
    alpha: f64,
    epsilon: f64,
) -> Fallible<f64> {
    let one_minus_delta = profile_one_minus_delta(profile, scale, epsilon)?;

    let base = positive_part(one_minus_delta.sub(Cert::point(alpha)?)?)?;

    if base.upper_f64()? == 0.0 {
        return Ok(0.0);
    }

    Cert::point(-epsilon)?
        .exp()?
        .mul(base)?
        .clamp01()?
        .lower_f64()
}
