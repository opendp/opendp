//! Batched RDP conversion. The optimizer only proposes orders. Every returned
//! bound is evaluated at a certified order and RDP upper bound, so neither
//! search truncation nor native rounding can invalidate privacy.
//!
//! The callback must upper-bound the same RDP curve at every requested order.
//! The true curve is nondecreasing, and K(t) = t * RDP(1 + t) is convex even
//! when the supplied upper bounds themselves are not convex.
//!
//! A separate large-delta search is unnecessary in the continuous-order
//! problem: for a sampled (alpha, gamma), monotonicity gives the constant
//! gamma bound at all smaller orders. Minimizing BC there yields at most
//! max(1/alpha, 1-exp(epsilon-gamma)), the large-delta theorem. The chords
//! after suffix minima are no worse than this constant bound. The first
//! knot is the smallest representable order above one, so orders between
//! it and one are not searched.

use crate::{
    error::{ErrorVariant, Fallible},
    traits::{Approximate, DirectedTranscendental, Interval, N64, SInterval, SoftFloat},
};

type I<S> = Interval<S>;

use super::{Bound, Direction, certify, evaluate_bound, evaluate_renyi, monotone};

#[derive(Clone, Copy)]
struct Knot {
    order: f64,
    gamma: f64,
}

// These are search parameters, not privacy or accuracy guarantees.
const ROUNDS: usize = 8;
const LOG_ORDER_STEP: i32 = 4;
const INITIAL_LOG_CAP: i32 = 8;
const MAX_EXPANSIONS: usize = 6;
const MAX_LOG_CAP: i32 = INITIAL_LOG_CAP << MAX_EXPANSIONS;
const MIN_LOG_WIDTH: f64 = 0.02;

/// Log-delta upper bounds at nondecreasing epsilon queries.
pub(crate) fn rdp_log_delta_batch(
    rdp: impl Fn(&[f64]) -> Fallible<Vec<f64>>,
    epsilon: &[f64],
) -> Fallible<Vec<f64>> {
    convert(&rdp, epsilon, Direction::Delta)
}

/// Epsilon upper bounds at nondecreasing log-delta targets.
pub(crate) fn rdp_epsilon_batch(
    rdp: impl Fn(&[f64]) -> Fallible<Vec<f64>>,
    log_delta: &[f64],
) -> Fallible<Vec<f64>> {
    convert(&rdp, log_delta, Direction::Epsilon)
}

fn convert(
    rdp: &impl Fn(&[f64]) -> Fallible<Vec<f64>>,
    queries: &[f64],
    direction: Direction,
) -> Fallible<Vec<f64>> {
    if queries.iter().any(|&q| {
        q.is_nan()
            || match direction {
                Direction::Delta => q < 0.0,
                Direction::Epsilon => q > 0.0,
            }
    }) || queries.windows(2).any(|w| w[0] > w[1])
    {
        return fallible!(
            FailedMap,
            "RDP conversion queries must be valid and nondecreasing"
        );
    }
    if queries.is_empty() {
        return Ok(Vec::new());
    }

    // Search both directions in increasing order of the query difficulty:
    // epsilon for log-delta, -log(delta) for epsilon.
    let targets: Vec<_> = match direction {
        Direction::Delta => queries.to_vec(),
        Direction::Epsilon => queries.iter().rev().map(|x| -x).collect(),
    };
    // Endpoint queries have no minimizing order and must not constrain
    // the monotone search for finite targets.
    let start = if matches!(direction, Direction::Epsilon) {
        targets.partition_point(|&target| target == 0.0)
    } else {
        0
    };
    let end = targets.partition_point(|target| target.is_finite());
    let search_targets = &targets[start..end];
    let mut knots = Vec::new();
    let mut log_cap = INITIAL_LOG_CAP;
    let mut orders = order_grid(-24, log_cap);
    orders.insert(0, 1.0_f64.next_up());
    let mut refinements = 0;
    let choices = loop {
        sample(rdp, &mut knots, &orders)?;
        if knots.iter().any(|k| k.gamma == 0.0) {
            let value = match direction {
                Direction::Delta => f64::NEG_INFINITY,
                Direction::Epsilon => 0.0,
            };
            return Ok(vec![value; queries.len()]);
        }

        // Find the minima separately: the minimum of the two bounds need
        // not itself have monotone minimizing orders.
        let choices = select(&knots, search_targets, direction);
        // Expand only while a selected order reaches the sampled upper
        // endpoint. Ordinary budgets should not query extreme Renyi orders.
        let last = knots[knots.len() - 1];
        if log_cap < MAX_LOG_CAP
            && last.gamma.is_finite()
            && choices.iter().flatten().any(|c| c.order == last.order)
        {
            let next_cap = (2 * log_cap).min(MAX_LOG_CAP);
            orders = order_grid(log_cap + LOG_ORDER_STEP, next_cap);
            log_cap = next_cap;
            continue;
        }
        if refinements == ROUNDS {
            break choices;
        }
        orders = refine(&knots, &choices);
        if orders.is_empty() {
            break choices;
        }
        refinements += 1;
    };

    let mut out = targets
        .iter()
        .enumerate()
        .map(|(i, &target)| {
            if matches!(direction, Direction::Epsilon) && target == 0.0 {
                return Ok(0.0);
            }
            if matches!(direction, Direction::Epsilon) && target.is_infinite() {
                return Ok(f64::INFINITY);
            }
            if matches!(direction, Direction::Delta) && target.is_infinite() {
                // Only a finite RDP bound certifies absolute continuity.
                return Ok(if knots.iter().any(|k| k.gamma.is_finite()) {
                    f64::NEG_INFINITY
                } else {
                    0.0
                });
            }

            let mut best = direction.vacuous();
            for (bound, choice) in [Bound::BalleCanonne, Bound::Asoodeh]
                .into_iter()
                .zip(choices[i - start])
            {
                let k = knots[choice.index];
                let gamma = if matches!(bound, Bound::BalleCanonne) {
                    // The chord is evaluated with certified interval arithmetic.
                    // Never certify using the native interpolated gamma.
                    match chord_gamma::<SoftFloat>(k, knots[choice.index + 1], choice.order) {
                        Ok(gamma) => gamma,
                        Err(error) if error.variant == ErrorVariant::Overflow => continue,
                        Err(error) => return Err(error),
                    }
                } else {
                    k.gamma
                };
                if gamma.is_finite() {
                    best = best.min(certify(
                        direction,
                        bound,
                        SInterval::point(choice.order)?,
                        SInterval::point(gamma)?,
                        target,
                    )?);
                }
            }
            Ok(best)
        })
        .collect::<Fallible<Vec<_>>>()?;

    if matches!(direction, Direction::Epsilon) {
        out.reverse();
    }
    // A bound at a smaller epsilon (respectively smaller delta) is also
    // valid at every subsequent query. Native search need not be monotone.
    monotone(&mut out);
    Ok(out)
}

fn order_grid(first: i32, last: i32) -> Vec<f64> {
    (first..=last)
        .step_by(LOG_ORDER_STEP as usize)
        .map(|i| 1.0 + 2f64.powi(i))
        .collect()
}

fn sample(
    rdp: &impl Fn(&[f64]) -> Fallible<Vec<f64>>,
    knots: &mut Vec<Knot>,
    orders: &[f64],
) -> Fallible<()> {
    let values = evaluate_renyi(rdp, orders)?;
    knots.extend(
        orders
            .iter()
            .zip(values)
            .map(|(&order, gamma)| Knot { order, gamma }),
    );
    knots.sort_by(|a, b| a.order.total_cmp(&b.order));
    // The true RDP curve is nondecreasing. Tighten upper bounds by suffix minima,
    // including values from earlier rounds.
    let mut gamma = f64::INFINITY;
    for knot in knots.iter_mut().rev() {
        gamma = gamma.min(knot.gamma);
        knot.gamma = gamma;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Choice {
    index: usize,
    order: f64,
}

fn select(knots: &[Knot], targets: &[f64], direction: Direction) -> Vec<[Choice; 2]> {
    let initial = Choice {
        index: 0,
        order: knots[0].order,
    };
    let mut rows = vec![[initial; 2]; targets.len()];
    for (column, bound) in [Bound::BalleCanonne, Bound::Asoodeh]
        .into_iter()
        .enumerate()
    {
        let n = knots.len() - (matches!(bound, Bound::BalleCanonne) as usize);
        let candidate = |j: usize, i: usize| {
            let k = knots[i];
            if matches!(bound, Bound::BalleCanonne) {
                chord_candidate(k, knots[i + 1], targets[j], direction)
            } else {
                (
                    k.order,
                    fast(bound, direction, k.order, k.gamma, targets[j]),
                )
            }
        };
        row_minima(&candidate, &mut rows, column, 0, targets.len(), 0, n - 1);
    }
    rows
}

// Monotone divide-and-conquer over ordered candidates, O(n log(m) + m).
// BC has decreasing differences in (order, epsilon); its inverse has
// increasing order as the requested log-delta magnitude increases. For
// Asoodeh the magnitude of d log(delta) / d epsilon = t/(1-exp(-t*epsilon))
// increases with t. After suffix minima its applicability boundary is
// ordered too. Native rounding may violate these properties, but only affects
// tightness.
fn row_minima(
    candidate: &impl Fn(usize, usize) -> (f64, f64),
    rows: &mut [[Choice; 2]],
    column: usize,
    lo: usize,
    hi: usize,
    first: usize,
    last: usize,
) {
    if lo == hi {
        return;
    }
    let mid = lo + (hi - lo) / 2;
    let mut index = first;
    let (mut order, mut value) = candidate(mid, first);
    for i in first + 1..=last {
        let (a, v) = candidate(mid, i);
        if v < value {
            index = i;
            order = a;
            value = v;
        }
    }
    rows[mid][column] = Choice { index, order };
    row_minima(candidate, rows, column, lo, mid, first, index);
    row_minima(candidate, rows, column, mid + 1, hi, index, last);
}

fn refine(knots: &[Knot], choices: &[[Choice; 2]]) -> Vec<f64> {
    let mut orders = Vec::new();
    for row in choices {
        for (column, choice) in row.iter().enumerate() {
            // Probe the selected chord or the intervals adjacent to the
            // selected finite-order candidate. Use geometric midpoints in t.
            let first = if column == 0 {
                choice.index
            } else {
                choice.index.saturating_sub(1)
            };
            let last = choice.index.min(knots.len() - 2);
            for i in first..=last {
                let lo = knots[i].order - 1.0;
                let hi = knots[i + 1].order - 1.0;
                if hi.ln() - lo.ln() <= MIN_LOG_WIDTH {
                    continue;
                }
                let order = 1.0 + lo.sqrt() * hi.sqrt();
                if order > knots[i].order && order < knots[i + 1].order {
                    orders.push(order);
                }
            }
        }
    }
    orders.sort_by(f64::total_cmp);
    orders.dedup();
    orders
}

// On a chord K(t) = s*t + b the BC log-delta objective is
// K(t) - t*epsilon + t*ln(t) - (1+t)*ln(1+t).
// Its stationary order is t = 1/expm1(s-epsilon).
// The inverse objective has derivative (ln(1+t)-b-x)/t^2,
// where x = -log(delta), hence t = expm1(b+x).
fn chord_candidate(lo: Knot, hi: Knot, x: f64, direction: Direction) -> (f64, f64) {
    let bound = Bound::BalleCanonne;
    let l = lo.order - 1.0;
    let h = hi.order - 1.0;
    if hi.gamma.is_finite() {
        let slope = hi.gamma + (l / (h - l)) * (hi.gamma - lo.gamma);
        let intercept = l * (lo.gamma - slope);
        let t = match direction {
            Direction::Delta if slope <= x => h,
            Direction::Delta => 1.0 / (slope - x).exp_m1(),
            Direction::Epsilon => (intercept + x).exp_m1(),
        };
        // The minimum on a chord is its stationary point, clamped to
        // the interval. Only the selected point needs objective evaluation.
        if !t.is_nan() {
            let order = (1.0 + t.clamp(l, h)).clamp(lo.order, hi.order);
            if let Ok(gamma) = chord_gamma::<N64<Approximate>>(lo, hi, order) {
                let score = fast(bound, direction, order, gamma, x);
                if score.is_finite() {
                    return (order, score);
                }
            }
        }
    }
    // Nonfinite bounds or numerical failure in the approximate search.
    let left = fast(bound, direction, lo.order, lo.gamma, x);
    let right = fast(bound, direction, hi.order, hi.gamma, x);
    if right < left {
        (hi.order, right)
    } else {
        (lo.order, left)
    }
}

fn chord_gamma<S: DirectedTranscendental>(lo: Knot, hi: Knot, order: f64) -> Fallible<f64> {
    if order == lo.order {
        return Ok(lo.gamma);
    }
    if order == hi.order {
        return Ok(hi.gamma);
    }
    let one = I::<S>::point(1.0)?;
    let l = (I::point(lo.order)? - one.clone())?;
    let h = (I::point(hi.order)? - one.clone())?;
    let t = (I::point(order)? - one)?;
    let w = ((t.clone() - l.clone())? / (h.clone() - l.clone())?)?;
    let left = ((I::point(1.0)? - w.clone())? * (l * I::point(lo.gamma)?)?)?;
    let right = (w * (h * I::point(hi.gamma)?)?)?;
    ((left + right)? / t)?.upper_f64()
}

// Approximate arithmetic only chooses candidates. An unavailable candidate
// need not participate in the search.
fn fast(bound: Bound, direction: Direction, a: f64, g: f64, x: f64) -> f64 {
    if !g.is_finite() || !x.is_finite() {
        return f64::INFINITY;
    }
    let evaluate = || -> Fallible<Option<f64>> {
        let value = evaluate_bound::<N64<Approximate>>(
            direction,
            bound,
            I::point(a)?,
            I::point(g)?,
            I::point(x)?,
        )?;
        value.map(|v| v.upper_f64()).transpose()
    };
    evaluate().ok().flatten().unwrap_or(f64::INFINITY)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn batch_gaussian_tightness_and_shared_callback() -> Fallible<()> {
        let calls = Cell::new(0);
        let rho = 0.05;
        let curve = |orders: &[f64]| {
            calls.set(calls.get() + 1);
            // The upward rounding here is required by the callback contract.
            Ok(orders.iter().map(|a| (rho * a).next_up()).collect())
        };
        let eps: Vec<_> = (1..=128).map(|i| i as f64 / 32.0).collect();
        let logs = rdp_log_delta_batch(curve, &eps)?;
        assert!(calls.get() <= ROUNDS + MAX_EXPANSIONS + 1);
        assert!(logs.windows(2).all(|w| w[0] >= w[1]));
        for (&e, &d) in eps.iter().zip(&logs) {
            // Compare against an independent, dense, fixed-order oracle.
            let reference = (1..=2000)
                .map(|i| 1.0 + i as f64 / 20.0)
                .map(|a| {
                    fast(Bound::BalleCanonne, Direction::Delta, a, rho * a, e).min(fast(
                        Bound::Asoodeh,
                        Direction::Delta,
                        a,
                        rho * a,
                        e,
                    ))
                })
                .fold(0.0, f64::min);
            assert!(d <= reference + 0.03, "{e}: {d} vs {reference}");
        }

        calls.set(0);
        let log_targets = [-1000.0, -100.0, -20.0, -5.0, -1.0, 0.0];
        let inverse = rdp_epsilon_batch(curve, &log_targets)?;
        assert!(calls.get() <= ROUNDS + MAX_EXPANSIONS + 1);
        assert!(inverse.windows(2).all(|w| w[0] >= w[1]));
        assert!(inverse[0].is_finite());
        for (&epsilon, &target) in inverse.iter().zip(&log_targets) {
            let certified = rdp_log_delta_batch(curve, &[epsilon])?[0];
            // Independent candidate discovery can be slightly looser than
            // the fixed-order inverse. This is a tightness check, not a proof.
            assert!(
                certified <= target + 0.1,
                "{epsilon}: {certified} vs {target}"
            );
        }
        Ok(())
    }

    #[test]
    fn ordered_minima_and_chord_enclosure() -> Fallible<()> {
        let knots: Vec<_> = [1.01, 1.2, 2.0, 4.0, 10.0, 30.0, 100.0]
            .into_iter()
            .map(|a| Knot {
                order: a,
                gamma: (a * 0.05_f64).next_up(),
            })
            .collect();
        let targets = [0.01, 0.1, 0.3, 0.5, 1.0, 5.0, 20.0];
        for direction in [Direction::Delta, Direction::Epsilon] {
            let selected = select(&knots, &targets, direction);
            for (&x, row) in targets.iter().zip(selected) {
                for (bound, choice) in [Bound::BalleCanonne, Bound::Asoodeh].into_iter().zip(row) {
                    let eval = |i: usize| {
                        let k = knots[i];
                        if matches!(bound, Bound::BalleCanonne) {
                            chord_candidate(k, knots[i + 1], x, direction).1
                        } else {
                            fast(bound, direction, k.order, k.gamma, x)
                        }
                    };
                    let n = knots.len() - (matches!(bound, Bound::BalleCanonne) as usize);
                    let best = (0..n).map(eval).fold(f64::INFINITY, f64::min);
                    assert_eq!(eval(choice.index), best);
                }
            }
        }
        // Independently compare the analytical candidate to a grid on each chord.
        for pair in knots.windows(2) {
            let (lo, hi) = (pair[0], pair[1]);
            for direction in [Direction::Delta, Direction::Epsilon] {
                for &target in &targets {
                    let (_, score) = chord_candidate(lo, hi, target, direction);
                    for i in 0..=32 {
                        let a = lo.order + (hi.order - lo.order) * (i as f64 / 32.0);
                        let gamma = chord_gamma::<N64<Approximate>>(lo, hi, a)?;
                        let reference = fast(Bound::BalleCanonne, direction, a, gamma, target);
                        assert!(score <= reference + 1e-10);
                    }
                }
            }
        }
        let gamma = chord_gamma::<SoftFloat>(knots[2], knots[3], 3.0)?;
        assert!(gamma >= 0.15);
        assert!(gamma < 0.18);
        Ok(())
    }

    #[test]
    fn constant_rdp_chord_recovers_large_delta_bound() -> Fallible<()> {
        // The optimal order can lie far below the order at which a
        // finite certificate is supplied. Chords cover the lower orders.
        let curve = |orders: &[f64]| Ok(vec![2.0; orders.len()]);
        for epsilon in [0.0_f64, 0.5, 1.0, 1.9] {
            let reference = (-(epsilon - 2.0).exp_m1()).ln();
            let bound = rdp_log_delta_batch(curve, &[epsilon])?[0];
            assert!((bound - reference).abs() < 1e-12);
        }
        for delta in [0.2_f64, 0.5, 0.9] {
            let reference = (2.0 + (-delta).ln_1p()).max(0.0);
            let bound = rdp_epsilon_batch(curve, &[delta.ln()])?[0];
            assert!((bound - reference).abs() < 1e-12);
        }
        Ok(())
    }

    #[test]
    fn extreme_order_and_near_one_coverage() -> Fallible<()> {
        let constant = |orders: &[f64]| Ok(vec![1.0; orders.len()]);
        assert!(rdp_log_delta_batch(constant, &[2.0])?[0] < -100.0);
        let near_one = |orders: &[f64]| {
            Ok(orders
                .iter()
                .map(|&a| {
                    if a == 1.0_f64.next_up() {
                        0.0
                    } else {
                        f64::INFINITY
                    }
                })
                .collect())
        };
        assert_eq!(
            rdp_log_delta_batch(near_one, &[0.0])?,
            vec![f64::NEG_INFINITY]
        );
        Ok(())
    }

    #[test]
    fn batch_boundaries_and_callback_contract() -> Fallible<()> {
        let zero = |orders: &[f64]| Ok(vec![0.0; orders.len()]);
        let infinite = |orders: &[f64]| Ok(vec![f64::INFINITY; orders.len()]);
        assert_eq!(rdp_log_delta_batch(zero, &[0.0])?, vec![f64::NEG_INFINITY]);
        assert_eq!(rdp_epsilon_batch(zero, &[f64::NEG_INFINITY])?, vec![0.0]);
        assert_eq!(rdp_log_delta_batch(infinite, &[1.0])?, vec![0.0]);
        assert_eq!(rdp_epsilon_batch(infinite, &[-1.0])?, vec![f64::INFINITY]);
        // Adding endpoints must not change the search for interior queries.
        let gaussian = |orders: &[f64]| Ok(orders.iter().map(|a| (0.05 * a).next_up()).collect());
        let log_delta = rdp_log_delta_batch(gaussian, &[2.0])?[0];
        assert_eq!(
            rdp_log_delta_batch(gaussian, &[2.0, f64::INFINITY, f64::INFINITY])?,
            vec![log_delta, f64::NEG_INFINITY, f64::NEG_INFINITY],
        );
        let epsilon = rdp_epsilon_batch(gaussian, &[-100.0])?[0];
        assert_eq!(
            rdp_epsilon_batch(
                gaussian,
                &[f64::NEG_INFINITY, f64::NEG_INFINITY, -100.0, 0.0]
            )?,
            vec![f64::INFINITY, f64::INFINITY, epsilon, 0.0],
        );
        assert!(rdp_log_delta_batch(zero, &[1.0, 0.0]).is_err());
        assert!(rdp_epsilon_batch(zero, &[f64::NAN]).is_err());
        assert!(rdp_log_delta_batch(|_| Ok(vec![]), &[0.0]).is_err());
        assert_eq!(
            rdp_log_delta_batch(|_| unreachable!(), &[])?,
            Vec::<f64>::new()
        );
        Ok(())
    }
}
