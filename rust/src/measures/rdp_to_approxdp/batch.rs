//! Batched RDP conversion. The optimizer only proposes orders. Every returned
//! bound is evaluated at a certified order and RDP upper bound, so neither
//! search truncation nor native rounding can invalidate privacy.
//!
//! The callback must upper-bound the same RDP curve at every requested order.
//! The true curve is nondecreasing, and K(t) = t * RDP(1 + t) is convex even
//! when the supplied upper bounds themselves are not convex.

use crate::{
    error::{ErrorVariant, Fallible},
    traits::SInterval as I,
};

use super::{rdp_epsilon0_log_on, rdp_epsilon1_log_on, rdp_log_delta0_on, rdp_log_delta1_on};

#[derive(Clone, Copy)]
enum Direction {
    Delta,
    Epsilon,
}

#[derive(Clone, Copy)]
enum Bound {
    Chord,
    Asoodeh,
    LargeDelta,
}

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
        return fallible!(FailedMap, "RDP conversion queries must be valid and nondecreasing");
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
    let mut knots = Vec::new();
    let mut log_cap = INITIAL_LOG_CAP;
    let mut orders = order_grid(-24, log_cap);
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

        // The minima are found separately: the minimum of the three bounds
        // need not itself have monotone minimizing orders.
        let choices = select(&knots, &targets, direction);
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
        .zip(choices)
        .map(|(&target, row)| {
            if matches!(direction, Direction::Epsilon) && target == 0.0 {
                return Ok(0.0);
            }
            if matches!(direction, Direction::Epsilon) && target.is_infinite() {
                return Ok(f64::INFINITY);
            }
            if matches!(direction, Direction::Delta) && target.is_infinite() {
                // Match the existing RDP conversion's extended-epsilon convention.
                return Ok(f64::NEG_INFINITY);
            }

            let mut best = match direction {
                Direction::Delta => 0.0,
                Direction::Epsilon => f64::INFINITY,
            };
            for (bound, choice) in [Bound::Chord, Bound::Asoodeh, Bound::LargeDelta]
                .into_iter()
                .zip(row)
            {
                let k = knots[choice.index];
                let gamma = if matches!(bound, Bound::Chord) {
                    // The chord is evaluated with certified interval arithmetic.
                    // Never certify using the native interpolated gamma.
                    match chord_gamma(k, knots[choice.index + 1], choice.order) {
                        Ok(gamma) => gamma,
                        Err(error) if error.variant == ErrorVariant::Overflow => continue,
                        Err(error) => return Err(error),
                    }
                } else {
                    k.gamma
                };
                best = best.min(certify(direction, bound, choice.order, gamma, target)?);
            }
            Ok(best)
        })
        .collect::<Fallible<Vec<_>>>()?;

    if matches!(direction, Direction::Epsilon) {
        out.reverse();
    }
    // A bound at a smaller epsilon (respectively smaller delta) is also
    // valid at every subsequent query. Native search need not be monotone.
    let mut best = f64::INFINITY;
    for value in &mut out {
        best = best.min(*value);
        *value = best;
    }
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
    let values = rdp(orders)?;
    if values.len() != orders.len() {
        return fallible!(FailedMap, "RDP callback output length must match input length");
    }
    if values.iter().any(|g| g.is_nan()) {
        return fallible!(NumericError, "RDP callback returned NaN");
    }
    if values.iter().any(|g| g.is_sign_negative()) {
        return fallible!(FailedMap, "RDP bounds must be nonnegative");
    }
    knots.extend(orders.iter().zip(values).map(|(&order, gamma)| Knot { order, gamma }));
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

fn select(knots: &[Knot], targets: &[f64], direction: Direction) -> Vec<[Choice; 3]> {
    let initial = Choice { index: 0, order: knots[0].order };
    let mut rows = vec![[initial; 3]; targets.len()];
    for (column, bound) in [Bound::Chord, Bound::Asoodeh, Bound::LargeDelta]
        .into_iter()
        .enumerate()
    {
        let n = knots.len() - (matches!(bound, Bound::Chord) as usize);
        let candidate = |j: usize, i: usize| {
            let k = knots[i];
            if matches!(bound, Bound::Chord) {
                chord_candidate(k, knots[i + 1], targets[j], direction)
            } else {
                (k.order, fast(bound, direction, k.order, k.gamma, targets[j]))
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
// ordered too. The large-delta envelope has a single ordered crossing.
// Native rounding may violate these properties, but only affects tightness.
fn row_minima(
    candidate: &impl Fn(usize, usize) -> (f64, f64),
    rows: &mut [[Choice; 3]],
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

fn refine(knots: &[Knot], choices: &[[Choice; 3]]) -> Vec<f64> {
    let mut orders = Vec::new();
    for row in choices {
        for (column, choice) in row.iter().enumerate() {
            // Probe the selected chord or the intervals adjacent to the
            // selected finite-order candidate. Use geometric midpoints in t.
            let first = if column == 0 { choice.index } else { choice.index.saturating_sub(1) };
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
    let l = lo.order - 1.0;
    let h = hi.order - 1.0;
    let mut order = lo.order;
    let mut score = fast(Bound::Chord, direction, lo.order, lo.gamma, x);
    let right = fast(Bound::Chord, direction, hi.order, hi.gamma, x);
    if right < score {
        order = hi.order;
        score = right;
    }
    if !hi.gamma.is_finite() {
        return (order, score);
    }
    let slope = hi.gamma + (l / (h - l)) * (hi.gamma - lo.gamma);
    let intercept = l * (lo.gamma - slope);
    let t = match direction {
        Direction::Delta if slope <= x => h,
        Direction::Delta => 1.0 / (slope - x).exp_m1(),
        Direction::Epsilon => (intercept + x).exp_m1(),
    };
    let a = 1.0 + t.clamp(l, h);
    if a > lo.order && a < hi.order {
        let w = (a - lo.order) / (hi.order - lo.order);
        let gamma = (1.0 - w) * (l / (a - 1.0)) * lo.gamma
            + w * (h / (a - 1.0)) * hi.gamma;
        let value = fast(Bound::Chord, direction, a, gamma, x);
        if value < score {
            return (a, value);
        }
    }
    (order, score)
}

fn chord_gamma(lo: Knot, hi: Knot, order: f64) -> Fallible<f64> {
    if order == lo.order {
        return Ok(lo.gamma);
    }
    if order == hi.order {
        return Ok(hi.gamma);
    }
    let one = I::point(1.0)?;
    let l = (I::point(lo.order)? - one.clone())?;
    let h = (I::point(hi.order)? - one.clone())?;
    let t = (I::point(order)? - one)?;
    let w = ((t.clone() - l.clone())? / (h.clone() - l.clone())?)?;
    let left = ((I::point(1.0)? - w.clone())? * (l * I::point(lo.gamma)?)?)?;
    let right = (w * (h * I::point(hi.gamma)?)?)?;
    ((left + right)? / t)?.upper_f64()
}

fn log_expm1(x: f64) -> f64 {
    if x < std::f64::consts::LN_2 {
        x.exp_m1().ln()
    } else {
        x + (-(-x).exp()).ln_1p()
    }
}

fn log1mexp(x: f64) -> f64 {
    if x < -std::f64::consts::LN_2 {
        (-x.exp()).ln_1p()
    } else {
        (-x.exp_m1()).ln()
    }
}

fn fast(bound: Bound, direction: Direction, a: f64, g: f64, x: f64) -> f64 {
    if !g.is_finite() || !x.is_finite() {
        return f64::INFINITY;
    }
    let t = a - 1.0;
    let correction = || a * (-1.0 / a).ln_1p() - t.ln();
    let value = match (direction, bound) {
        (Direction::Delta, Bound::Chord) => t * (g - x) + correction(),
        (Direction::Delta, Bound::Asoodeh) if x > g => {
            log_expm1(t * g) - a.ln() - log_expm1(t * x)
        }
        (Direction::Delta, Bound::LargeDelta) if x >= g => -a.ln(),
        (Direction::Delta, Bound::LargeDelta) => (-a.ln()).max(log1mexp(x - g)),
        (Direction::Epsilon, Bound::Chord) => g + (correction() + x) / t,
        (Direction::Epsilon, Bound::Asoodeh) if x > a.ln() => {
            let z = log_expm1(t * g) - a.ln() + x;
            (z.max(0.0) + (-z.abs()).exp().ln_1p()) / t
        }
        (Direction::Epsilon, Bound::LargeDelta) if x <= a.ln() => g + log1mexp(-x),
        _ => f64::INFINITY,
    };
    if value.is_nan() { f64::INFINITY } else { value }
}

// Certify the selected order for each theorem. A loose search or chord
// can only loosen the resulting privacy bound.
fn certify(direction: Direction, bound: Bound, a: f64, g: f64, x: f64) -> Fallible<f64> {
    let vacuous = match direction {
        Direction::Delta => 0.0,
        Direction::Epsilon => f64::INFINITY,
    };
    if !g.is_finite() {
        return Ok(vacuous);
    }
    let best = match certify_bound(direction, bound, a, g, x) {
        Ok(Some(value)) => vacuous.min(value),
        Ok(None) => vacuous,
        Err(error) if error.variant == ErrorVariant::Overflow => vacuous,
        Err(error) => return Err(error),
    };
    Ok(match direction {
        // A finite-order positive RDP bound cannot certify exact zero delta.
        Direction::Delta => best.max(-f64::MAX),
        Direction::Epsilon => best.max(0.0),
    })
}

fn certify_bound(
    direction: Direction,
    bound: Bound,
    a: f64,
    g: f64,
    x: f64,
) -> Fallible<Option<f64>> {
    let (a, g, x) = (I::point(a)?, I::point(g)?, I::point(x)?);
    let value = match (direction, bound) {
        (Direction::Delta, Bound::Chord) => Some(rdp_log_delta0_on(a, g, x)?),
        (Direction::Delta, Bound::Asoodeh) => rdp_log_delta1_on(a, g, x)?,
        (Direction::Delta, Bound::LargeDelta) => {
            let floor = -a.ln()?;
            Some(if x.lower_f64()? >= g.upper_f64()? {
                floor
            } else {
                let gap = (x - g)?;
                floor.max((-gap.exp_m1()?).ln()?)
            })
        }
        (Direction::Epsilon, Bound::Chord) => Some(rdp_epsilon0_log_on(a, g, -x)?),
        (Direction::Epsilon, Bound::Asoodeh) => rdp_epsilon1_log_on(a, g, -x)?,
        (Direction::Epsilon, Bound::LargeDelta) => {
            // Need delta >= 1/alpha. Check in log space, including rounding.
            if (a.ln()? - x.clone())?.lower_f64()? < 0.0 {
                None
            } else {
                Some((g + (-(-x).exp_m1()?).ln()?)?)
            }
        }
    };
    value.map(|v| v.upper_f64()).transpose()
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
                    fast(Bound::Chord, Direction::Delta, a, rho * a, e)
                        .min(fast(Bound::Asoodeh, Direction::Delta, a, rho * a, e))
                        .min(fast(Bound::LargeDelta, Direction::Delta, a, rho * a, e))
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
            assert!(certified <= target + 0.1, "{epsilon}: {certified} vs {target}");
        }
        Ok(())
    }

    #[test]
    fn ordered_minima_and_chord_enclosure() -> Fallible<()> {
        let knots: Vec<_> = [1.01, 1.2, 2.0, 4.0, 10.0, 30.0, 100.0]
            .into_iter()
            .map(|a| Knot { order: a, gamma: (a * 0.05_f64).next_up() })
            .collect();
        let targets = [0.01, 0.1, 0.3, 0.5, 1.0, 5.0, 20.0];
        for direction in [Direction::Delta, Direction::Epsilon] {
            let selected = select(&knots, &targets, direction);
            for (&x, row) in targets.iter().zip(selected) {
                for (bound, choice) in [Bound::Chord, Bound::Asoodeh, Bound::LargeDelta]
                    .into_iter().zip(row)
                {
                    let eval = |i: usize| {
                        let k = knots[i];
                        if matches!(bound, Bound::Chord) {
                            chord_candidate(k, knots[i + 1], x, direction).1
                        } else {
                            fast(bound, direction, k.order, k.gamma, x)
                        }
                    };
                    let n = knots.len() - (matches!(bound, Bound::Chord) as usize);
                    let best = (0..n).map(eval).fold(f64::INFINITY, f64::min);
                    assert_eq!(eval(choice.index), best);
                }
            }
        }
        let gamma = chord_gamma(knots[2], knots[3], 3.0)?;
        assert!(gamma >= 0.15);
        assert!(gamma < 0.18);
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
        assert!(rdp_log_delta_batch(zero, &[1.0, 0.0]).is_err());
        assert!(rdp_epsilon_batch(zero, &[f64::NAN]).is_err());
        assert!(rdp_log_delta_batch(|_| Ok(vec![]), &[0.0]).is_err());
        assert_eq!(rdp_log_delta_batch(|_| unreachable!(), &[])?, Vec::<f64>::new());
        Ok(())
    }
}
