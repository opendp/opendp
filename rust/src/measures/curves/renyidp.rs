use std::sync::Arc;

use crate::{
    error::{ErrorVariant, Fallible},
    measures::{
        curves::{
            PrivacyProfile, RenyiFn, check_alpha, check_epsilon,
            logspace::{check_delta, delta_to_log_lower_unchecked, log_to_delta_upper},
            profile_to_tradeoff::beta_via_profile,
        },
        rdp_to_approxdp::batch::{rdp_epsilon_batch, rdp_log_delta_batch},
    },
    traits::{CInterval, SInterval},
    utilities::search::{OptimizationResult, SearchMode, fallible_optimize_unimodal},
};

type Cert = SInterval;

// 1.0 + sqrt(f64::EPSILON). Written as a literal because f64::sqrt is not
// always const-stable on older MSRVs.
const ALPHA_MIN: f64 = 1.0000000149011612;

// The largest order whose square remains representable in f64. The certified
// kernels use alpha^2 in their high-order regime, so this is a backend-derived
// endpoint rather than an arbitrary optimization cap.
const ALPHA_MAX: f64 = 1.3407807929942596e154;
const ALPHA_START_CAP: f64 = 2.0;

// The beta envelope is not assumed globally unimodal. These constants are only
// for locating a finite cap; missing later orders remains conservative.
const BETA_GRID_PER_DOUBLING: usize = 16;
const BETA_LOCAL_GRID: usize = 32;
const BETA_STALE_DOUBLINGS: usize = 2;

// Safe-direction slack for f64 values returned by closures and native search
// arithmetic. Formal conservativeness comes from recomputing final formulas
// with certified intervals where it matters.
const ROUNDING_SLACK: f64 = 128.0 * f64::EPSILON;

/// Optimize Rényi order in log space. The grid and local-refinement policy is
/// intentionally local to RDP conversion: the objective may have kinks or
/// multiple extrema, and the result is only a candidate for the certified
/// recomputation below.
fn optimize_log_order<F>(
    mode: SearchMode,
    arg_lo: f64,
    arg_hi: f64,
    local_grid: usize,
    f_arg: F,
) -> Fallible<OptimizationResult>
where
    F: Fn(f64) -> Fallible<f64>,
{
    if !arg_lo.is_finite()
        || !arg_hi.is_finite()
        || arg_lo <= 0.0
        || arg_hi <= 0.0
        || arg_lo > arg_hi
    {
        return fallible!(
            Search,
            "RDP order bounds must be finite, positive, and ordered"
        );
    }
    if arg_lo == arg_hi {
        return Ok(OptimizationResult {
            arg: arg_lo,
            value: rdp_objective_value(f_arg(arg_lo)?)?,
        });
    }

    let x_lo = arg_lo.ln();
    let x_hi = arg_hi.ln();
    let evaluate = |x: f64| {
        let arg = x.exp().clamp(arg_lo, arg_hi);
        rdp_objective_value(f_arg(arg)?).map(|value| (arg, value))
    };

    if local_grid < 3 {
        let optimum =
            fallible_optimize_unimodal(mode, x_lo, x_hi, |x| evaluate(x).map(|(_, value)| value))?;
        return Ok(OptimizationResult {
            arg: optimum.arg.exp().clamp(arg_lo, arg_hi),
            value: optimum.value,
        });
    }

    let grid = local_grid.max(3);
    let mut xs = Vec::with_capacity(grid);
    let mut values = Vec::with_capacity(grid);
    let mut best = OptimizationResult {
        arg: arg_lo,
        value: evaluate(x_lo)?.1,
    };

    for i in 0..grid {
        let x = interpolate_log(x_lo, x_hi, i as f64 / (grid - 1) as f64);
        let (arg, value) = evaluate(x)?;
        xs.push(x);
        values.push(value);
        if mode.is_better(value, best.value) {
            best = OptimizationResult { arg, value };
        }
    }

    for i in 0..grid {
        let no_worse = match mode {
            SearchMode::Minimize => {
                (i == 0 || values[i] <= values[i - 1])
                    && (i + 1 == grid || values[i] <= values[i + 1])
            }
            SearchMode::Maximize => {
                (i == 0 || values[i] >= values[i - 1])
                    && (i + 1 == grid || values[i] >= values[i + 1])
            }
        };
        let strict_edge = match mode {
            SearchMode::Minimize => {
                (i > 0 && values[i] < values[i - 1]) || (i + 1 < grid && values[i] < values[i + 1])
            }
            SearchMode::Maximize => {
                (i > 0 && values[i] > values[i - 1]) || (i + 1 < grid && values[i] > values[i + 1])
            }
        };
        if !(no_worse && strict_edge) {
            continue;
        }

        let left = if i == 0 { xs[i] } else { xs[i - 1] };
        let right = if i + 1 == grid { xs[i] } else { xs[i + 1] };
        if right <= left {
            continue;
        }

        let candidate =
            fallible_optimize_unimodal(mode, left, right, |x| evaluate(x).map(|(_, value)| value))?;
        if mode.is_better(candidate.value, best.value) {
            best = OptimizationResult {
                arg: candidate.arg.exp().clamp(arg_lo, arg_hi),
                value: candidate.value,
            };
        }
    }

    Ok(best)
}

fn rdp_objective_value(value: f64) -> Fallible<f64> {
    if value.is_nan() {
        return fallible!(NumericError, "RDP objective value must not be NaN");
    }
    Ok(value)
}

fn sample_log_order<F>(
    mode: SearchMode,
    arg_lo: f64,
    arg_hi: f64,
    grid: usize,
    f_arg: F,
) -> Fallible<OptimizationResult>
where
    F: Fn(f64) -> Fallible<f64>,
{
    debug_assert!(arg_lo.is_finite() && arg_hi.is_finite() && arg_lo > 0.0 && arg_lo <= arg_hi);
    let x_lo = arg_lo.ln();
    let x_hi = arg_hi.ln();
    let mut best = OptimizationResult {
        arg: arg_lo,
        value: rdp_objective_value(f_arg(arg_lo)?)?,
    };

    for i in 0..grid.max(2) {
        let arg = interpolate_log(x_lo, x_hi, i as f64 / (grid.max(2) - 1) as f64)
            .exp()
            .clamp(arg_lo, arg_hi);
        let value = rdp_objective_value(f_arg(arg)?)?;
        if mode.is_better(value, best.value) {
            best = OptimizationResult { arg, value };
        }
    }
    Ok(best)
}

fn interpolate_log(lo: f64, hi: f64, fraction: f64) -> f64 {
    lo + fraction * (hi - lo)
}

/// Convert an RDP profile to an approximate-DP delta upper bound.
///
/// Contract on `rdp`: for every finite alpha > 1 passed to it, return a
/// conservative upper bound on the RDP epsilon at that order.
#[allow(non_snake_case)]
pub fn delta_via_renyiDP<R>(curve: R, source_delta: f64, epsilon: f64) -> Fallible<f64>
where
    R: Fn(&[f64]) -> Fallible<Vec<f64>>,
{
    Ok(delta_via_renyiDP_batch(curve, source_delta, &[epsilon])?[0])
}

/// Convert sorted epsilon queries using a shared set of RDP orders.
#[allow(non_snake_case)]
pub(super) fn delta_via_renyiDP_batch<R>(
    curve: R,
    source_delta: f64,
    epsilons: &[f64],
) -> Fallible<Vec<f64>>
where
    R: Fn(&[f64]) -> Fallible<Vec<f64>>,
{
    check_delta(source_delta)?;
    super::check_inputs(epsilons, check_epsilon)?;
    let n = epsilons.partition_point(|e| e.is_finite());
    let mut deltas = rdp_log_delta_batch(&curve, &epsilons[..n])?
        .into_iter()
        .map(|log_delta| {
            add_approximate_rdp_delta(log_to_delta_upper(log_delta)?, source_delta)
        })
        .collect::<Fallible<Vec<_>>>()?;
    deltas.resize(epsilons.len(), source_delta);
    Ok(deltas)
}

/// Invert sorted ordinary-delta budgets directly at certified Renyi orders.
#[allow(non_snake_case)]
pub(super) fn epsilon_via_renyiDP_batch<R>(
    curve: R,
    source_delta: f64,
    deltas: &[f64],
) -> Fallible<Vec<f64>>
where
    R: Fn(&[f64]) -> Fallible<Vec<f64>>,
{
    check_delta(source_delta)?;
    super::check_inputs(deltas, check_delta)?;
    let mut epsilons = vec![f64::INFINITY; deltas.len()];
    let start = deltas.partition_point(|&d| d < source_delta);
    let end = deltas.partition_point(|&d| d < 1.0);
    epsilons[end..].fill(0.0);
    if start >= end {
        return Ok(epsilons);
    }
    let mut targets = Vec::with_capacity(end - start);
    for &delta in &deltas[start..end] {
        targets.push(if delta == source_delta {
            f64::NEG_INFINITY
        } else if source_delta == 0.0 {
            delta_to_log_lower_unchecked(delta)?
        } else {
            (SInterval::point(delta)? - SInterval::point(source_delta)?)?
                .ln()?
                .lower_f64()?
        });
    }
    epsilons[start..end].copy_from_slice(&rdp_epsilon_batch(curve, &targets)?);
    Ok(epsilons)
}

/// Convert an RDP profile to an f-DP tradeoff lower bound beta(alpha).
///
/// Here `alpha` is the type-I error in the f-DP tradeoff function.
/// Contract on `rdp`: for every finite order > 1 passed to it, return a
/// conservative upper bound on the RDP epsilon at that order.
#[allow(non_snake_case)]
pub fn beta_via_renyiDP(curve: Arc<RenyiFn>, source_delta: f64, alpha: f64) -> Fallible<f64> {
    check_delta(source_delta)?;
    check_alpha(alpha)?;

    if source_delta != 0.0 {
        let profile = PrivacyProfile::new_log(move |epsilons| {
            rdp_log_delta_batch(curve.as_ref(), epsilons)
        })
        .with_delta_slack(source_delta)?;
        return beta_via_profile(&profile, alpha);
    }

    if alpha == 0.0 {
        // Every valid finite-RDP guarantee has the trivial beta(0)=1
        // endpoint. Keep this representation-specific rather than making
        // PrivacyGuarantee bypass callback and conversion representations.
        return Ok(1.0);
    }
    if alpha == 1.0 {
        return Ok(0.0);
    }

    let rdp = |order| evaluate_renyi(curve.as_ref(), &[order]).map(|values| values[0]);
    let alpha_cap = find_beta_alpha_cap(alpha, &rdp)?;
    beta_via_renyiDP_core(alpha, alpha_cap, &rdp)
}

/// Apply the delta parameter from an approximate-RDP statement to the
/// conversion theorem's delta bound. This addition is local to RDP semantics.
fn add_approximate_rdp_delta(conversion_delta: f64, source_delta: f64) -> Fallible<f64> {
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

// -----------------------------------------------------------------------------
// Bounded cores
// -----------------------------------------------------------------------------

#[allow(non_snake_case)]
fn beta_via_renyiDP_core<R>(alpha: f64, alpha_cap: f64, rdp: &R) -> Fallible<f64>
where
    R: Fn(f64) -> Fallible<f64>,
{
    let alpha_cap = normalize_alpha_cap(alpha_cap);

    let optimum = optimize_log_order(
        SearchMode::Maximize,
        ALPHA_MIN,
        alpha_cap,
        BETA_LOCAL_GRID,
        |order| {
            let gamma = rdp_upper(rdp, order)?;
            if gamma == 0.0 {
                return Ok(1.0 - alpha);
            }
            if gamma.is_infinite() {
                return Ok(0.0);
            }

            beta_at_order_lower(alpha, order, gamma)
        },
    )?;

    let beta = if optimum.value.is_finite() {
        optimum.value
    } else {
        0.0
    };

    Ok(beta.clamp(0.0, 1.0 - alpha))
}

// -----------------------------------------------------------------------------
// Beta cap finder
// -----------------------------------------------------------------------------

fn find_beta_alpha_cap<R>(alpha: f64, rdp: &R) -> Fallible<f64>
where
    R: Fn(f64) -> Fallible<f64>,
{
    let objective = |order: f64| -> Fallible<f64> {
        let gamma = rdp_upper(rdp, order)?;
        beta_at_order_lower(alpha, order, gamma)
    };

    let mut best = 0f64;
    let mut stale = 0usize;

    let mut lo = ALPHA_MIN;
    let mut hi = ALPHA_START_CAP;

    loop {
        let before = best;
        let band_best = sample_log_order(
            SearchMode::Maximize,
            lo,
            hi,
            BETA_GRID_PER_DOUBLING,
            &objective,
        )?
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
            return Ok(hi);
        }

        if hi >= ALPHA_MAX {
            return Ok(hi);
        }

        lo = hi;
        hi = (hi * 2.0).min(ALPHA_MAX);
    }
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
        (LogTerm::Finite(x), LogTerm::Finite(y)) => (x.exp()? + y.exp()?)?.ln()?,
    };

    let div = (log_sum / Cert::point(order - 1.0)?)?;

    Ok(div.lower_f64()?.max(0.0))
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

    let sum = (Cert::point(t1)? + Cert::point(t2)?)?;
    Ok(sum.lower_f64()?.max(0.0))
}

fn kl_term(p: f64, q: f64) -> Fallible<f64> {
    if p == 0.0 {
        return Ok(0.0);
    }
    if q == 0.0 {
        return Ok(f64::INFINITY);
    }

    let out = (Cert::point(p)? * (Cert::point(p)?.ln()? - Cert::point(q)?.ln()?)?)?;

    out.lower_f64()
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
        (LogTerm::Finite(x), LogTerm::Finite(y)) => Ok(LogTerm::Finite((x + y)?)),
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
        (Cert::point(coeff)? * Cert::point(p)?.ln()?)?,
    ))
}

// -----------------------------------------------------------------------------
// Numeric helpers
// -----------------------------------------------------------------------------

/// Validate whole callback batches without changing their numerical bounds.
/// The existing scalar kernels apply their directed widening in `rdp_upper`.
pub(super) fn evaluate_renyi<R>(curve: &R, orders: &[f64]) -> Fallible<Vec<f64>>
where
    R: Fn(&[f64]) -> Fallible<Vec<f64>> + ?Sized,
{
    for &order in orders {
        if !(order > 1.0) || !order.is_finite() {
            return fallible!(
                FailedMap,
                "alpha ({}) must be finite and greater than 1",
                order
            );
        }
    }
    if orders.windows(2).any(|pair| pair[0] > pair[1]) {
        return fallible!(FailedMap, "RDP orders must be nondecreasing");
    }
    let values = curve(orders)?;
    if values.len() != orders.len() {
        return fallible!(
            FailedMap,
            "RDP callback output length ({}) must match input length ({})",
            values.len(),
            orders.len()
        );
    }
    for &value in &values {
        if value.is_nan() {
            return fallible!(NumericError, "RDP value must not be NaN");
        }
        if value.is_sign_negative() {
            return fallible!(FailedMap, "RDP value ({}) must be non-negative", value);
        }
    }
    Ok(values)
}

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
        return fallible!(NumericError, "RDP value must not be NaN");
    }
    if value.is_sign_negative() {
        return fallible!(FailedMap, "RDP value ({}) must be non-negative", value);
    }
    if value == 0.0 || value.is_infinite() {
        return Ok(value);
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
        alpha_cap.min(ALPHA_MAX)
    } else {
        ALPHA_MAX
    }
}

fn prob_next_down(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }

    (x - ROUNDING_SLACK * x.abs().max(1.0)).next_down()
}

fn improvement_threshold(a: f64, b: f64) -> f64 {
    ROUNDING_SLACK * a.abs().max(b.abs()).max(1.0)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Error;

    fn assert_later_callback_error_propagates(
        make_error: fn() -> Error,
        expected_variant: ErrorVariant,
    ) {
        let curve: Arc<RenyiFn> = Arc::new(move |orders| {
            orders
                .iter()
                .map(|&order| {
                    if order <= 2.0 {
                        Ok(1.0)
                    } else {
                        Err(make_error())
                    }
                })
                .collect()
        });

        for result in [
            delta_via_renyiDP(curve.as_ref(), 0.0, 1.0),
            beta_via_renyiDP(curve.clone(), 0.0, 0.25),
        ] {
            let error = result.unwrap_err();
            assert_eq!(error.variant, expected_variant);
            assert_eq!(error.message.as_deref(), Some("later order failed"));
        }
    }

    #[test]
    fn test_later_callback_numeric_error_propagates() {
        assert_later_callback_error_propagates(
            || err!(NumericError, "later order failed"),
            ErrorVariant::NumericError,
        );
    }

    #[test]
    fn test_later_callback_function_error_propagates() {
        assert_later_callback_error_propagates(
            || err!(FailedFunction, "later order failed"),
            ErrorVariant::FailedFunction,
        );
    }

    #[test]
    fn test_later_callback_overflow_propagates() {
        // Callback errors propagate independently of numerical search.
        assert_later_callback_error_propagates(
            || err!(Overflow, "later order failed"),
            ErrorVariant::Overflow,
        );
    }

    #[test]
    fn test_nan_objectives_propagate() {
        for mode in [SearchMode::Minimize, SearchMode::Maximize] {
            for result in [
                optimize_log_order(mode, ALPHA_MIN, 2.0, 4, |_| Ok(f64::NAN)),
                sample_log_order(mode, ALPHA_MIN, 2.0, 4, |_| Ok(f64::NAN)),
            ] {
                assert_eq!(result.unwrap_err().variant, ErrorVariant::NumericError);
            }
        }
    }

    #[test]
    fn test_direct_inverse_preserves_source_slack() -> Fallible<()> {
        let curve = |orders: &[f64]| Ok(vec![0.0; orders.len()]);
        let source = 0.125;
        assert_eq!(
            epsilon_via_renyiDP_batch(curve, source, &[0.0, source, 1.0])?,
            vec![f64::INFINITY, 0.0, 0.0]
        );
        assert_eq!(
            delta_via_renyiDP_batch(curve, source, &[0.0, 1.0, f64::INFINITY])?,
            vec![source; 3]
        );
        Ok(())
    }
}
