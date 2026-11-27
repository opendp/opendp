use std::sync::Arc;

use crate::{
    error::{ErrorVariant, Fallible},
    measures::{
        curves::{
            PrivacyProfile, RenyiFn, add_log_delta_slack, check_alpha, check_epsilon,
            invert_delta_budgets,
            logspace::{check_delta, log_to_delta_upper},
            maximize::{Candidate, refine_maxima},
            profile_to_tradeoff::beta_via_profile_batch,
        },
        rdp_to_approxdp::batch::{rdp_epsilon_batch, rdp_log_delta_batch},
    },
    traits::SInterval,
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
const BETA_STALE_DOUBLINGS: usize = 2;

// Heuristic threshold for stopping order expansion.
const IMPROVEMENT_SLACK: f64 = 128.0 * f64::EPSILON;

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
    rdp_log_delta_batch(&curve, epsilons)?
        .into_iter()
        .map(|log_delta| log_to_delta_upper(log_delta, source_delta))
        .collect()
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
    invert_delta_budgets(deltas, source_delta, false, |targets| {
        rdp_epsilon_batch(curve, targets)
    })
}

#[allow(non_snake_case)]
pub(super) fn epsilon_from_log_delta_via_renyiDP_batch<R>(
    curve: R,
    source_delta: f64,
    log_deltas: &[f64],
) -> Fallible<Vec<f64>>
where
    R: Fn(&[f64]) -> Fallible<Vec<f64>>,
{
    check_delta(source_delta)?;
    invert_delta_budgets(log_deltas, source_delta, true, |targets| {
        rdp_epsilon_batch(curve, targets)
    })
}

#[allow(non_snake_case)]
pub(super) fn log_delta_via_renyiDP_batch<R>(
    curve: R,
    source_delta: f64,
    epsilons: &[f64],
) -> Fallible<Vec<f64>>
where
    R: Fn(&[f64]) -> Fallible<Vec<f64>>,
{
    check_delta(source_delta)?;
    super::check_inputs(epsilons, check_epsilon)?;
    rdp_log_delta_batch(curve, epsilons)?
        .into_iter()
        .map(|core| add_log_delta_slack(core, source_delta))
        .collect()
}

/// Convert an RDP profile into f-DP lower bounds, sharing sampled Renyi
/// orders across all type-I errors. Search scores only select candidates;
/// each result uses a certified Bernoulli bound at its selected order.
#[allow(non_snake_case)]
pub(super) fn beta_via_renyiDP_batch(
    curve: Arc<RenyiFn>,
    source_delta: f64,
    alphas: &[f64],
) -> Fallible<Vec<f64>> {
    check_delta(source_delta)?;
    super::check_inputs(alphas, check_alpha)?;
    if source_delta != 0.0 {
        let profile =
            PrivacyProfile::new_log(move |epsilons| rdp_log_delta_batch(curve.as_ref(), epsilons))
                .with_delta_slack(source_delta)?;
        return beta_via_profile_batch(&profile, alphas);
    }

    let start = alphas.partition_point(|&alpha| alpha == 0.0);
    let end = alphas.partition_point(|&alpha| alpha < 1.0);
    let mut out = vec![0.0; alphas.len()];
    if start > 0 {
        let endpoint_curve = curve.clone();
        let profile = PrivacyProfile::new_log(move |epsilons| {
            rdp_log_delta_batch(endpoint_curve.as_ref(), epsilons)
        });
        out[..start].copy_from_slice(&beta_via_profile_batch(&profile, &alphas[..start])?);
    }
    if start == end {
        return Ok(out);
    }
    let alphas = &alphas[start..end];

    // Search in log-order coordinates. Every source evaluation is sorted
    // and deduplicated in these coordinates.
    let order = |x: f64| x.exp().clamp(ALPHA_MIN, ALPHA_MAX);
    let evaluate = |xs: &[f64]| {
        let orders = xs.iter().map(|&x| order(x)).collect::<Vec<_>>();
        evaluate_renyi(curve.as_ref(), &orders)
    };
    let score =
        |query: usize, x: f64, gamma: f64| beta_at_order_fast(alphas[query], order(x), gamma);

    let mut rows = vec![Vec::<Candidate>::new(); alphas.len()];
    let mut maxima = vec![0.0_f64; alphas.len()];
    let mut stale = vec![0; alphas.len()];
    let mut lo = ALPHA_MIN.ln();
    let mut hi = ALPHA_START_CAP.ln();
    let cap = ALPHA_MAX.ln();
    loop {
        let points = (0..BETA_GRID_PER_DOUBLING)
            .filter(|&i| rows[0].is_empty() || i != 0)
            .map(|i| {
                let fraction = i as f64 / (BETA_GRID_PER_DOUBLING - 1) as f64;
                if fraction == 1.0 {
                    hi
                } else {
                    lo + fraction * (hi - lo)
                }
            })
            .collect::<Vec<_>>();
        let bounds = evaluate(&points)?;
        for (query, row) in rows.iter_mut().enumerate() {
            let before = maxima[query];
            for (&arg, &bound) in points.iter().zip(&bounds) {
                let value = score(query, arg, bound)?;
                maxima[query] = maxima[query].max(value);
                row.push(Candidate {
                    arg,
                    bound,
                    score: value,
                });
            }
            if maxima[query] > before + improvement_threshold(before, maxima[query]) {
                stale[query] = 0;
            } else {
                stale[query] += 1;
            }
        }
        if stale.iter().all(|&count| count >= BETA_STALE_DOUBLINGS) || hi == cap {
            break;
        }
        lo = hi;
        hi = (hi + std::f64::consts::LN_2).min(cap);
    }

    // Refine strict local maxima (including endpoints), without assuming
    // that the whole order envelope is unimodal.
    let mut best = Vec::with_capacity(alphas.len());
    let mut brackets = Vec::new();
    for (query, row) in rows.iter().enumerate() {
        let mut candidate = row[0];
        for (i, &point) in row.iter().enumerate() {
            if point.score > candidate.score {
                candidate = point;
            }
            let left = row[i.saturating_sub(1)];
            let right = row[(i + 1).min(row.len() - 1)];
            if point.score >= left.score
                && point.score >= right.score
                && (point.score > left.score || point.score > right.score)
                && left.arg < right.arg
            {
                brackets.push((query, left.arg, right.arg));
            }
        }
        best.push(candidate);
    }
    refine_maxima(brackets, &mut best, evaluate, score)?;
    for (i, (&alpha, candidate)) in alphas.iter().zip(best).enumerate() {
        out[start + i] = beta_at_order_lower(alpha, order(candidate.arg), candidate.bound)?;
    }
    let mut best = 0.0_f64;
    for value in out.iter_mut().rev() {
        best = best.max(*value);
        *value = best;
    }
    Ok(out)
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
    let complement = (Cert::point(1.0)? - Cert::point(type_i)?)?.lower_f64()?;
    if gamma == 0.0 {
        return Ok(complement);
    }
    if gamma.is_infinite() {
        return Ok(0.0);
    }

    // Only points proved infeasible advance the lower endpoint.
    search_beta(complement, |beta| {
        bernoulli_region_maybe_contains(type_i, beta, order, gamma)
    })
}

// Native log-space Bernoulli calculation used only for candidate selection.
fn beta_at_order_fast(type_i: f64, order: f64, gamma: f64) -> Fallible<f64> {
    let complement = 1.0 - type_i;
    if gamma == 0.0 {
        return Ok(complement);
    }
    if gamma.is_infinite() {
        return Ok(0.0);
    }

    let t = order - 1.0;
    let p = [type_i.ln(), (-type_i).ln_1p()];
    let renyi = |p: [f64; 2], q: [f64; 2]| {
        let a = p[0] + t * (p[0] - q[0]);
        let b = p[1] + t * (p[1] - q[1]);
        (a.max(b) + (a.min(b) - a.max(b)).exp().ln_1p()) / t
    };
    search_beta(complement, |beta| {
        let q = [(-beta).ln_1p(), beta.ln()];
        Ok(renyi(p, q) <= gamma && renyi(q, p) <= gamma)
    })
}

fn search_beta(mut hi: f64, maybe_contains: impl Fn(f64) -> Fallible<bool>) -> Fallible<f64> {
    let mut lo = 0.0;
    loop {
        let mid = lo + (hi - lo) / 2.0;
        if mid == lo || mid == hi {
            break;
        }
        if maybe_contains(mid)? {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(lo)
}

fn bernoulli_region_maybe_contains(
    type_i: f64,
    beta: f64,
    order: f64,
    gamma_upper: f64,
) -> Fallible<bool> {
    let one = Cert::point(1.0)?;
    let p = Cert::point(type_i)?;
    let beta = Cert::point(beta)?;
    let p_bar = (one.clone() - p.clone())?;
    let q = (one - beta.clone())?;

    let d_pq = bernoulli_renyi_lower([p.clone(), p_bar.clone()], [q.clone(), beta.clone()], order)?;
    let d_qp = bernoulli_renyi_lower([q, beta], [p, p_bar], order)?;
    Ok(d_pq <= gamma_upper && d_qp <= gamma_upper)
}

fn bernoulli_renyi_lower(p: [Cert; 2], q: [Cert; 2], order: f64) -> Fallible<f64> {
    let [p1, p2] = p;
    let [q1, q2] = q;
    let order = Cert::point(order)?;
    let t = (order.clone() - Cert::point(1.0)?)?;
    let t1 = bernoulli_log_term(order.clone(), t.clone(), p1, q1)?;
    let t2 = bernoulli_log_term(order, t.clone(), p2, q2)?;

    let log_sum = match (t1, t2) {
        (LogTerm::Infinity, _) | (_, LogTerm::Infinity) => return Ok(f64::INFINITY),
        (LogTerm::Zero, LogTerm::Zero) => return Ok(0.0),
        (LogTerm::Zero, LogTerm::Finite(x)) | (LogTerm::Finite(x), LogTerm::Zero) => x,
        (LogTerm::Finite(x), LogTerm::Finite(y)) => {
            let pivot = x.clone().max(y.clone());
            let gap = (x.min(y) - pivot.clone())?;
            if gap.lower_f64()? < -1024.0 {
                // The larger log term alone is a lower bound on the sum.
                pivot
            } else {
                (pivot + gap.exp()?.ln_1p()?)?
            }
        }
    };
    Ok((log_sum / t)?.lower_f64()?.max(0.0))
}

enum LogTerm {
    Zero,
    Infinity,
    Finite(Cert),
}

fn bernoulli_log_term(order: Cert, t: Cert, p: Cert, q: Cert) -> Fallible<LogTerm> {
    if p.upper_f64()? == 0.0 {
        return Ok(LogTerm::Zero);
    }
    if q.upper_f64()? == 0.0 {
        return Ok(LogTerm::Infinity);
    }
    // log(p^order * q^(1-order)); neither probability was rounded to f64
    // before its complement or logarithm was computed.
    Ok(LogTerm::Finite(((order * p.ln()?)? - (t * q.ln()?)?)?))
}

// -----------------------------------------------------------------------------
// Numeric helpers
// -----------------------------------------------------------------------------

/// Tight RDP bound implied by pure DP (the binary randomized-response extremum).
/// Factor exp(alpha * epsilon) from the numerator to avoid positive
/// exponential overflow. Pure DP itself supplies a fallback upper bound.
#[allow(non_snake_case)]
pub(crate) fn rdp_from_pureDP(alpha: f64, epsilon: f64) -> Fallible<f64> {
    if !alpha.is_finite() || alpha <= 1.0 {
        return fallible!(FailedMap, "Rényi order must be finite and greater than one");
    }
    check_epsilon(epsilon)?;
    if epsilon == 0.0 || epsilon >= 64.0 {
        return Ok(epsilon);
    }

    let evaluate = || -> Fallible<f64> {
        let eps = Cert::point(epsilon)?;
        let t = (Cert::point(alpha)? - Cert::point(1.0)?)?;
        let exponent = -((Cert::point(2.0)? * t.clone())? + Cert::point(1.0)?)?;
        let numerator = (exponent * eps.clone())?.exp()?.ln_1p()?;
        let denominator = (-eps.clone()).exp()?.ln_1p()?;
        (eps + ((numerator - denominator)? / t)?)?.upper_f64()
    };
    // Failure of the optional numerical refinement cannot invalidate epsilon-DP.
    Ok(evaluate().unwrap_or(epsilon).clamp(0.0, epsilon))
}

pub(super) use crate::measures::rdp_to_approxdp::evaluate_renyi;

fn improvement_threshold(a: f64, b: f64) -> f64 {
    IMPROVEMENT_SLACK * a.abs().max(b.abs()).max(1.0)
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
            beta_via_renyiDP_batch(curve.clone(), 0.0, &[0.25]).map(|values| values[0]),
        ] {
            let error = result.unwrap_err();
            assert_eq!(error.variant, expected_variant);
            assert_eq!(error.message.as_deref(), Some("later order failed"));
        }
    }

    #[test]
    fn pure_dp_renyi_matches_randomized_response() -> Fallible<()> {
        for (alpha, epsilon) in [(1.5_f64, 0.5_f64), (2.0, 1.0), (8.0, 0.25)] {
            let expected =
                ((alpha * epsilon).exp() + ((1.0 - alpha) * epsilon).exp()) / (epsilon.exp() + 1.0);
            let expected = expected.ln() / (alpha - 1.0);
            let bound = rdp_from_pureDP(alpha, epsilon)?;
            assert!((bound - expected).abs() < 1e-13);
            assert!(bound <= epsilon);
        }
        assert_eq!(rdp_from_pureDP(2.0, 0.0)?, 0.0);
        assert!(rdp_from_pureDP(f64::MAX, f64::MAX)?.is_finite());
        Ok(())
    }

    #[test]
    fn bernoulli_complements_preserve_small_probabilities() -> Fallible<()> {
        assert_eq!(
            beta_at_order_lower(2.0_f64.powi(-54), 2.0, 0.0)?,
            1.0_f64.next_down()
        );
        assert!(bernoulli_region_maybe_contains(
            0.25,
            2.0_f64.powi(-60),
            2.0,
            100.0
        )?);
        for (type_i, order, gamma) in [(0.25, 2.0, 0.1), (0.01, 1.5, 0.3), (0.9, 64.0, 1.0)] {
            let fast = beta_at_order_fast(type_i, order, gamma)?;
            let certified = beta_at_order_lower(type_i, order, gamma)?;
            assert!((fast - certified).abs() < 1e-10);
        }
        Ok(())
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
    fn test_beta_grid_batches_orders() -> Fallible<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let count = Arc::new(AtomicUsize::new(0));
        let recorded = count.clone();
        let curve: Arc<RenyiFn> = Arc::new(move |orders: &[f64]| {
            recorded.fetch_add(1, Ordering::Relaxed);
            assert!(orders.windows(2).all(|w| w[0] <= w[1]));
            Ok(orders.iter().map(|&order| 0.1 * order).collect())
        });
        let alphas = [0.125, 0.25, 0.5, 0.75];
        let values = beta_via_renyiDP_batch(curve.clone(), 0.0, &alphas)?;
        let batch_calls = count.swap(0, Ordering::Relaxed);
        for (&alpha, value) in alphas.iter().zip(values) {
            let singleton = beta_via_renyiDP_batch(curve.clone(), 0.0, &[alpha])?[0];
            assert!(value > 0.0);
            assert!((value - singleton).abs() < 1e-8);
        }
        assert!(batch_calls < count.load(Ordering::Relaxed));
        Ok(())
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
