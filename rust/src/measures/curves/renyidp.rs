use crate::{
    error::Fallible,
    measures::curves::{check_alpha, check_epsilon},
    traits::DInterval,
    utilities::search::{SearchMode, optimize_log_domain_to_precision, sample_log_domain},
};

type Cert = DInterval;

// 1.0 + sqrt(f64::EPSILON). Written as a literal because f64::sqrt is not
// always const-stable on older MSRVs.
const ALPHA_MIN: f64 = 1.0000000149011612;

// sqrt(f64::MAX). Avoids alpha * alpha style overflow in fast search code.
const ALPHA_HARD_CAP: f64 = 1.3407807929942596e154;
const ALPHA_START_CAP: f64 = 2.0;

// The final delta search is over the minimum of several per-order upper
// bounds. Each bound is well-behaved, but the minimum can have kinks, so use a
// small log-order grid and refine local minima.
const DELTA_LOCAL_GRID: usize = 32;

// The beta envelope is not assumed globally unimodal. These constants are only
// for locating a finite cap; missing later orders remains conservative.
const BETA_GRID_PER_DOUBLING: usize = 16;
const BETA_LOCAL_GRID: usize = 32;
const BETA_STALE_DOUBLINGS: usize = 2;

// Safe-direction slack for f64 values returned by closures and native search
// arithmetic. Formal conservativeness comes from recomputing final formulas
// with DInterval where it matters.
const ROUNDING_SLACK: f64 = 128.0 * f64::EPSILON;

// ln(2^-1074), the smallest positive f64 subnormal.
const LOG_TRUE_MIN: f64 = -744.4400719213812;

/// Convert an RDP profile to an approximate-DP delta upper bound.
///
/// Contract on `rdp`: for every finite alpha > 1 passed to it, return a
/// conservative upper bound on the RDP epsilon at that order.
#[allow(non_snake_case)]
pub fn delta_via_renyiDP<R>(curve: R, epsilon: f64) -> Fallible<f64>
where
    R: Fn(f64) -> Fallible<f64>,
{
    check_epsilon(epsilon)?;

    if epsilon.is_infinite() {
        return Ok(0.0);
    }

    let alpha_cap = find_delta_alpha_cap(&curve, epsilon);
    delta_via_renyiDP_core(&curve, epsilon, alpha_cap)
}

/// Convert an RDP profile to an f-DP tradeoff lower bound beta(alpha).
///
/// Here `alpha` is the type-I error in the f-DP tradeoff function.
/// Contract on `rdp`: for every finite order > 1 passed to it, return a
/// conservative upper bound on the RDP epsilon at that order.
#[allow(non_snake_case)]
pub fn beta_via_renyiDP<R>(curve: R, alpha: f64) -> Fallible<f64>
where
    R: Fn(f64) -> Fallible<f64>,
{
    check_alpha(alpha)?;

    if alpha == 1.0 {
        return Ok(0.0);
    }

    let alpha_cap = find_beta_alpha_cap(alpha, &curve);
    beta_via_renyiDP_core(alpha, alpha_cap, &curve)
}

/// Convert rho-zCDP to an approximate-DP delta upper bound.
#[allow(non_snake_case)]
pub fn delta_via_zCDP(rho: f64, epsilon: f64) -> Fallible<f64> {
    check_rho(rho)?;
    check_epsilon(epsilon)?;

    if rho == 0.0 || epsilon.is_infinite() {
        return Ok(0.0);
    }

    if rho.is_infinite() {
        return Ok(1.0);
    }

    let alpha_cap = zcdp_delta_alpha_cap(rho, epsilon);
    let curve = |alpha: f64| zcdp_rdp_upper(rho, alpha);

    delta_via_renyiDP_core(&curve, epsilon, alpha_cap)
}

/// Convert rho-zCDP to an f-DP tradeoff lower bound beta(alpha).
///
/// Here `alpha` is the type-I error in the f-DP tradeoff function.
#[allow(non_snake_case)]
pub fn beta_via_zCDP(rho: f64, alpha: f64) -> Fallible<f64> {
    check_rho(rho)?;
    check_alpha(alpha)?;

    if alpha == 0.0 {
        return Ok(1.0);
    }
    if alpha == 1.0 {
        return Ok(0.0);
    }
    if rho == 0.0 {
        return Ok(1.0 - alpha);
    }
    if rho.is_infinite() {
        return Ok(0.0);
    }

    let rdp = |order: f64| zcdp_rdp_upper(rho, order);
    let alpha_cap = find_beta_alpha_cap(alpha, &rdp);

    beta_via_renyiDP_core(alpha, alpha_cap, &rdp)
}

// -----------------------------------------------------------------------------
// Bounded cores
// -----------------------------------------------------------------------------

#[allow(non_snake_case)]
fn delta_via_renyiDP_core<R>(curve: &R, epsilon: f64, alpha_cap: f64) -> Fallible<f64>
where
    R: Fn(f64) -> Fallible<f64>,
{
    let alpha_cap = normalize_alpha_cap(alpha_cap);

    let optimum = optimize_log_domain_to_precision(
        SearchMode::Minimize,
        ALPHA_MIN,
        alpha_cap,
        Some(DELTA_LOCAL_GRID),
        |order| {
            let gamma = match rdp_upper(curve, order) {
                Ok(v) => v,
                Err(_) => return f64::INFINITY,
            };
            log_delta_fast(order, gamma, epsilon)
        },
    );

    let alpha = optimum.arg;
    let gamma = rdp_upper(curve, alpha)?;
    let log_delta = log_delta_upper_at_order(alpha, gamma, epsilon)?;

    log_delta_to_delta_upper(log_delta)
}

#[allow(non_snake_case)]
fn beta_via_renyiDP_core<R>(alpha: f64, alpha_cap: f64, rdp: &R) -> Fallible<f64>
where
    R: Fn(f64) -> Fallible<f64>,
{
    let alpha_cap = normalize_alpha_cap(alpha_cap);

    let optimum = optimize_log_domain_to_precision(
        SearchMode::Maximize,
        ALPHA_MIN,
        alpha_cap,
        Some(BETA_LOCAL_GRID),
        |order| {
            let gamma = match rdp_upper(rdp, order) {
                Ok(v) if v == 0.0 => return 1.0 - alpha,
                Ok(v) if v.is_finite() => v,
                Ok(_) => return 0.0,
                Err(_) => return 0.0,
            };

            beta_at_order_lower(alpha, order, gamma).unwrap_or(0.0)
        },
    );

    let beta = if optimum.value.is_finite() {
        optimum.value
    } else {
        0.0
    };

    Ok(beta.clamp(0.0, 1.0 - alpha))
}

// -----------------------------------------------------------------------------
// Delta cap finders
// -----------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum DeltaBranch {
    BalleCanonne,
    Asoodeh,
    LargeDelta,
}

fn find_delta_alpha_cap<R>(curve: &R, epsilon: f64) -> f64
where
    R: Fn(f64) -> Fallible<f64>,
{
    let mut cap: f64 = ALPHA_START_CAP;

    for branch in [
        DeltaBranch::BalleCanonne,
        DeltaBranch::Asoodeh,
        DeltaBranch::LargeDelta,
    ] {
        cap = cap.max(find_delta_branch_cap(epsilon, curve, branch));
    }

    normalize_alpha_cap(cap)
}

fn find_delta_branch_cap<R>(epsilon: f64, rdp: &R, branch: DeltaBranch) -> f64
where
    R: Fn(f64) -> Fallible<f64>,
{
    let objective = |order: f64| -> f64 {
        let gamma = match rdp_upper(rdp, order) {
            Ok(v) => v,
            Err(_) => return f64::INFINITY,
        };

        match branch {
            DeltaBranch::BalleCanonne => log_delta_balle_canonne_fast(order, gamma, epsilon),
            DeltaBranch::Asoodeh => log_delta_asoodeh_fast(order, gamma, epsilon),
            DeltaBranch::LargeDelta => log_delta_large_delta_fast(order, gamma, epsilon),
        }
    };

    let mut curr_order = ALPHA_START_CAP;

    let mut prev_value = objective(ALPHA_MIN);
    let mut curr_value = objective(curr_order);

    loop {
        if !strictly_smaller(curr_value, prev_value) {
            return curr_order;
        }

        if curr_order >= ALPHA_HARD_CAP / 2.0 {
            return ALPHA_HARD_CAP;
        }

        prev_value = curr_value;

        curr_order *= 2.0;
        curr_value = objective(curr_order);
    }
}

fn zcdp_delta_alpha_cap(rho: f64, epsilon: f64) -> f64 {
    // Branch 0 has its zCDP optimum no larger than roughly this value.
    let branch0_cap = ((epsilon + 1.0) / (2.0 * rho) + 4.0).next_up();

    // Branch 1 is useful only while epsilon > alpha * rho. Include the crossing
    // point so the bounded search covers the whole useful branch-1 domain.
    let branch1_cap = if epsilon > rho {
        (epsilon / rho).next_up()
    } else {
        ALPHA_START_CAP
    };

    normalize_alpha_cap(branch0_cap.max(branch1_cap))
}

// -----------------------------------------------------------------------------
// Beta cap finder
// -----------------------------------------------------------------------------

fn find_beta_alpha_cap<R>(alpha: f64, rdp: &R) -> f64
where
    R: Fn(f64) -> Fallible<f64>,
{
    let objective = |order: f64| -> f64 {
        let gamma = match rdp_upper(rdp, order) {
            Ok(v) if v == 0.0 => return 1.0 - alpha,
            Ok(v) if v.is_finite() => v,
            Ok(_) => return 0.0,
            Err(_) => return 0.0,
        };

        beta_at_order_lower(alpha, order, gamma).unwrap_or(0.0)
    };

    let mut best = 0f64;
    let mut stale = 0usize;

    let mut lo = ALPHA_MIN;
    let mut hi = ALPHA_START_CAP;

    loop {
        let before = best;
        let band_best = sample_log_domain(
            SearchMode::Maximize,
            lo,
            hi,
            BETA_GRID_PER_DOUBLING,
            &objective,
        )
        .value;
        best = best.max(if band_best.is_finite() {
            band_best
        } else {
            0.0
        });

        if best > before + improvement_threshold(before, best) {
            stale = 0;
        } else {
            stale += 1;
        }

        if stale >= BETA_STALE_DOUBLINGS {
            return hi;
        }

        if hi >= ALPHA_HARD_CAP / 2.0 {
            return ALPHA_HARD_CAP;
        }

        lo = hi;
        hi *= 2.0;
    }
}

// -----------------------------------------------------------------------------
// Delta kernels
// -----------------------------------------------------------------------------

fn log_delta_upper_at_order(alpha: f64, gamma: f64, epsilon: f64) -> Fallible<f64> {
    if !(alpha > 1.0) || !alpha.is_finite() {
        return fallible!(
            FailedMap,
            "alpha ({}) must be finite and greater than 1",
            alpha
        );
    }

    if gamma == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if gamma.is_infinite() {
        return Ok(0.0);
    }

    let b0 = log_delta_balle_canonne_upper(alpha, gamma, epsilon).unwrap_or(f64::INFINITY);
    let b1 = log_delta_asoodeh_upper(alpha, gamma, epsilon).unwrap_or(f64::INFINITY);
    let b2 = log_delta_large_delta_upper(alpha, gamma, epsilon).unwrap_or(f64::INFINITY);

    Ok(clean_log_delta(b0.min(b1).min(b2)))
}

fn log_delta_balle_canonne_upper(alpha: f64, gamma: f64, epsilon: f64) -> Fallible<f64> {
    let a = Cert::point(alpha)?;
    let one = Cert::point(1.0)?;
    let gamma = Cert::point(gamma)?;
    let eps = Cert::point(epsilon)?;

    let a_m1 = a.clone().sub(one.clone())?;
    let gamma_minus_eps = gamma.sub(eps)?;
    let t1 = a_m1.clone().mul(gamma_minus_eps)?;

    // alpha * ln(1 - 1 / alpha)
    let one_over_alpha = one.clone().div(a.clone())?;
    let log_inner = one.sub(one_over_alpha)?.ln()?;
    let t2 = a.mul(log_inner)?;

    let out = t1.add(t2)?.sub(a_m1.ln()?)?;

    Ok(out.upper_f64().unwrap_or(f64::INFINITY))
}

fn log_delta_asoodeh_upper(alpha: f64, gamma: f64, epsilon: f64) -> Fallible<f64> {
    // Useful only when epsilon > gamma. If this check fails because `gamma` is
    // rounded upward, skipping the branch is conservative.
    if !(epsilon > gamma) || gamma == 0.0 || epsilon == 0.0 {
        return Ok(f64::INFINITY);
    }

    let a = Cert::point(alpha)?;
    let one = Cert::point(1.0)?;
    let gamma = Cert::point(gamma)?;
    let eps = Cert::point(epsilon)?;
    let a_m1 = a.clone().sub(one)?;

    let num_log = a_m1.clone().mul(gamma)?.exp_m1()?.ln()?;
    let den_log = a_m1.mul(eps)?.exp_m1()?.ln()?;
    let out = num_log.sub(a.ln()?)?.sub(den_log)?;

    Ok(out.upper_f64().unwrap_or(f64::INFINITY))
}

fn log_delta_large_delta_upper(alpha: f64, gamma: f64, epsilon: f64) -> Fallible<f64> {
    let a = Cert::point(alpha)?;
    let log_one_over_alpha = a.clone().ln()?.neg()?;

    if epsilon >= gamma {
        return Ok(log_one_over_alpha.upper_f64().unwrap_or(f64::INFINITY));
    }

    // log(1 - exp(epsilon - gamma)) = log(-expm1(epsilon - gamma)).
    let z = Cert::point(epsilon)?.sub(Cert::point(gamma)?)?;
    let needed = Cert::point(0.0)?.sub(z.exp_m1()?)?.ln()?;

    let out = log_one_over_alpha.max(needed)?;
    Ok(out.upper_f64().unwrap_or(f64::INFINITY))
}

fn log_delta_fast(alpha: f64, gamma: f64, epsilon: f64) -> f64 {
    clean_log_delta(
        log_delta_balle_canonne_fast(alpha, gamma, epsilon)
            .min(log_delta_asoodeh_fast(alpha, gamma, epsilon))
            .min(log_delta_large_delta_fast(alpha, gamma, epsilon)),
    )
}

fn log_delta_balle_canonne_fast(alpha: f64, gamma: f64, epsilon: f64) -> f64 {
    if !(alpha > 1.0) || gamma.is_nan() || gamma.is_sign_negative() {
        return f64::INFINITY;
    }
    if gamma == 0.0 {
        return f64::NEG_INFINITY;
    }
    if gamma.is_infinite() {
        return 0.0;
    }

    let a_m1 = alpha - 1.0;

    a_m1 * (gamma - epsilon) + alpha * (-1.0 / alpha).ln_1p() - a_m1.ln()
}

fn log_delta_asoodeh_fast(alpha: f64, gamma: f64, epsilon: f64) -> f64 {
    if !(alpha > 1.0) || !(epsilon > gamma) || gamma == 0.0 || epsilon == 0.0 {
        return f64::INFINITY;
    }

    let a_m1 = alpha - 1.0;
    log_expm1_pos_fast(a_m1 * gamma) - alpha.ln() - log_expm1_pos_fast(a_m1 * epsilon)
}

fn log_delta_large_delta_fast(alpha: f64, gamma: f64, epsilon: f64) -> f64 {
    if !(alpha > 1.0) || gamma.is_nan() || gamma.is_sign_negative() {
        return f64::INFINITY;
    }
    if gamma == 0.0 {
        return f64::NEG_INFINITY;
    }
    if gamma.is_infinite() {
        return 0.0;
    }

    let log_one_over_alpha = -alpha.ln();

    if epsilon >= gamma {
        return log_one_over_alpha;
    }

    log_one_over_alpha.max(log1mexp_fast(epsilon - gamma))
}

// -----------------------------------------------------------------------------
// Beta / f-DP kernel
// -----------------------------------------------------------------------------

fn beta_at_order_lower(type_i: f64, order: f64, gamma: f64) -> Fallible<f64> {
    debug_assert!((0.0..=1.0).contains(&type_i));
    debug_assert!(order > 1.0);
    debug_assert!(gamma >= 0.0);

    if type_i == 0.0 {
        // This value is usually not tight, but it is always conservative and
        // avoids endpoint singularities in the Bernoulli region.
        return Ok(0.0);
    }
    if type_i == 1.0 {
        return Ok(0.0);
    }
    if gamma == 0.0 {
        return Ok(1.0 - type_i);
    }
    if gamma.is_infinite() {
        return Ok(0.0);
    }

    let mut lo = 0.0;
    let mut hi = 1.0 - type_i;

    loop {
        let mid = lo + (hi - lo) / 2.0;

        if mid == lo || mid == hi {
            break;
        }

        if bernoulli_region_maybe_contains(type_i, mid, order, gamma)? {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    // `lo` is kept on the conservative side of the boundary. Nudge downward.
    Ok(prob_next_down(lo).clamp(0.0, 1.0 - type_i))
}

fn bernoulli_region_maybe_contains(
    type_i: f64,
    beta: f64,
    order: f64,
    gamma_upper: f64,
) -> Fallible<bool> {
    let p = type_i;
    let q = 1.0 - beta;

    // Use lower bounds on the Bernoulli Renyi divergences and an upper bound
    // on gamma. This can create false positives, which only lowers the returned
    // beta and is conservative. It should not create false negatives.
    let d_pq_lo = bernoulli_renyi_lower(p, q, order)?;
    let d_qp_lo = bernoulli_renyi_lower(q, p, order)?;

    let gamma = gamma_upper_next(gamma_upper);

    Ok(d_pq_lo <= gamma && d_qp_lo <= gamma)
}

fn bernoulli_renyi_lower(p: f64, q: f64, order: f64) -> Fallible<f64> {
    debug_assert!((0.0..=1.0).contains(&p));
    debug_assert!((0.0..=1.0).contains(&q));
    debug_assert!(order > 1.0);

    if p == q {
        return Ok(0.0);
    }

    if (order - 1.0).abs() <= 1e-6 {
        return bernoulli_kl_lower(p, q);
    }

    let t1 = bernoulli_log_term(order, p, q)?;
    let t2 = bernoulli_log_term(order, 1.0 - p, 1.0 - q)?;

    let log_sum = match (t1, t2) {
        (LogTerm::Infinity, _) | (_, LogTerm::Infinity) => return Ok(f64::INFINITY),
        (LogTerm::Zero, LogTerm::Zero) => return Ok(0.0),
        (LogTerm::Zero, LogTerm::Finite(x)) | (LogTerm::Finite(x), LogTerm::Zero) => x,
        (LogTerm::Finite(x), LogTerm::Finite(y)) => x.exp()?.add(y.exp()?)?.ln()?,
    };

    let div = log_sum.div(Cert::point(order - 1.0)?)?;

    Ok(div.lower_f64().unwrap_or(f64::INFINITY).max(0.0))
}

fn bernoulli_kl_lower(p: f64, q: f64) -> Fallible<f64> {
    if p == q {
        return Ok(0.0);
    }

    let t1 = kl_term(p, q)?;
    let t2 = kl_term(1.0 - p, 1.0 - q)?;

    if t1.is_infinite() || t2.is_infinite() {
        return Ok(f64::INFINITY);
    }

    let sum = Cert::point(t1)?.add(Cert::point(t2)?)?;
    Ok(sum.lower_f64().unwrap_or(f64::INFINITY).max(0.0))
}

fn kl_term(p: f64, q: f64) -> Fallible<f64> {
    if p == 0.0 {
        return Ok(0.0);
    }
    if q == 0.0 {
        return Ok(f64::INFINITY);
    }

    let out = Cert::point(p)?.mul(Cert::point(p)?.ln()?.sub(Cert::point(q)?.ln()?)?)?;

    Ok(out.lower_f64().unwrap_or(f64::INFINITY))
}

#[derive(Clone)]
enum LogTerm {
    Zero,
    Infinity,
    Finite(Cert),
}

fn bernoulli_log_term(order: f64, p: f64, q: f64) -> Fallible<LogTerm> {
    // log(p^order * q^(1 - order))
    let a = log_scaled_prob(order, p)?;
    let b = log_scaled_prob(1.0 - order, q)?;

    match (a, b) {
        (LogTerm::Zero, _) | (_, LogTerm::Zero) => Ok(LogTerm::Zero),
        (LogTerm::Infinity, _) | (_, LogTerm::Infinity) => Ok(LogTerm::Infinity),
        (LogTerm::Finite(x), LogTerm::Finite(y)) => Ok(LogTerm::Finite(x.add(y)?)),
    }
}

fn log_scaled_prob(coeff: f64, p: f64) -> Fallible<LogTerm> {
    debug_assert!((0.0..=1.0).contains(&p));

    if coeff == 0.0 {
        return Ok(LogTerm::Finite(Cert::point(0.0)?));
    }

    if p == 0.0 {
        return if coeff > 0.0 {
            Ok(LogTerm::Zero)
        } else {
            Ok(LogTerm::Infinity)
        };
    }

    Ok(LogTerm::Finite(
        Cert::point(coeff)?.mul(Cert::point(p)?.ln()?)?,
    ))
}

// -----------------------------------------------------------------------------
// Numeric helpers
// -----------------------------------------------------------------------------

fn rdp_upper<R>(rdp: &R, alpha: f64) -> Fallible<f64>
where
    R: Fn(f64) -> Fallible<f64>,
{
    if !(alpha > 1.0) || !alpha.is_finite() {
        return fallible!(
            FailedMap,
            "alpha ({}) must be finite and greater than 1",
            alpha
        );
    }

    let value = rdp(alpha)?;

    if value.is_nan() {
        return fallible!(FailedMap, "RDP value must not be NaN");
    }
    if value.is_sign_negative() {
        return fallible!(FailedMap, "RDP value ({}) must be non-negative", value);
    }
    if value == 0.0 || value.is_infinite() {
        return Ok(value);
    }

    Ok(value.next_up())
}

fn zcdp_rdp_upper(rho: f64, alpha: f64) -> Fallible<f64> {
    let value = alpha * rho;

    if value == 0.0 {
        return Ok(0.0);
    }
    if value.is_infinite() {
        return Ok(f64::INFINITY);
    }
    if value.is_nan() || value.is_sign_negative() {
        return fallible!(FailedMap, "computed zCDP RDP value is invalid");
    }

    Ok(value.next_up())
}

fn gamma_upper_next(gamma: f64) -> f64 {
    if gamma == 0.0 || gamma.is_infinite() {
        gamma
    } else {
        gamma.next_up()
    }
}

fn normalize_alpha_cap(alpha_cap: f64) -> f64 {
    if alpha_cap.is_finite() && alpha_cap > ALPHA_MIN {
        alpha_cap.min(ALPHA_HARD_CAP)
    } else {
        ALPHA_HARD_CAP
    }
}

fn clean_log_delta(value: f64) -> f64 {
    if value.is_nan() {
        f64::INFINITY
    } else if value > 0.0 {
        0.0
    } else {
        value
    }
}

fn log_delta_to_delta_upper(log_delta: f64) -> Fallible<f64> {
    if log_delta.is_nan() {
        return fallible!(FailedMap, "computed log(delta) is NaN");
    }

    if log_delta >= 0.0 {
        return Ok(1.0);
    }

    if log_delta == f64::NEG_INFINITY {
        return Ok(0.0);
    }

    let adjusted = log_next_up(log_delta);

    if adjusted <= LOG_TRUE_MIN {
        // The mathematical value may be positive even though f64::exp would
        // underflow. Return the smallest positive f64 instead of zero.
        return Ok(f64::from_bits(1));
    }

    let delta = Cert::point(adjusted)?
        .exp()?
        .upper_f64()
        .unwrap_or(f64::INFINITY);
    Ok(delta.next_up().clamp(0.0, 1.0))
}

fn log_expm1_pos_fast(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x < std::f64::consts::LN_2 {
        x.exp_m1().ln()
    } else {
        x + (-(-x).exp()).ln_1p()
    }
}

fn log1mexp_fast(x: f64) -> f64 {
    debug_assert!(x <= 0.0);

    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x < -std::f64::consts::LN_2 {
        (-x.exp()).ln_1p()
    } else {
        (-x.exp_m1()).ln()
    }
}

fn log_next_up(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }

    (x + ROUNDING_SLACK * x.abs().max(1.0)).next_up()
}

fn prob_next_down(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }

    (x - ROUNDING_SLACK * x.abs().max(1.0)).next_down()
}

fn strictly_smaller(a: f64, b: f64) -> bool {
    if !b.is_finite() && a.is_finite() {
        return true;
    }
    b - a > improvement_threshold(a, b)
}

fn improvement_threshold(a: f64, b: f64) -> f64 {
    ROUNDING_SLACK * a.abs().max(b.abs()).max(1.0)
}

fn check_rho(rho: f64) -> Fallible<()> {
    if rho.is_nan() {
        return fallible!(FailedMap, "rho must not be NaN");
    }
    if rho.is_sign_negative() {
        return fallible!(FailedMap, "rho ({}) must be non-negative", rho);
    }
    Ok(())
}
