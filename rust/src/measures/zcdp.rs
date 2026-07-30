// Copyright (c) 2022 President and Fellows of Harvard College
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright 2020 Thomas Steinke
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use opendp_derive::proven;

use crate::{
    error::Fallible,
    measures::rdp_to_approxdp::{
        rdp_epsilon0_on, rdp_epsilon1_on, rdp_log_delta0_on, rdp_log_delta1_on,
    },
    traits::{A, D, InfExp, Interval, IntervalArithmeticBackend, IntervalExpBackend},
    utilities::search::{BracketedOptimum, SearchMode, optimize_to_precision_bracket},
};

#[cfg(test)]
pub(crate) mod test;

// sqrt(f64::MAX). The cap keeps native search arithmetic away from alpha^2
// overflow while remaining far beyond practical optima.
const ALPHA_HARD_CAP: f64 = 1.3407807929942596e154;

/// Return a conservative upper bound on `log(delta)` for rho-zCDP at epsilon.
///
/// Ordinary native interval arithmetic (`Interval<A>`) is used only to select
/// promising Renyi orders. Each candidate order is then recomputed with
/// `Interval<D>`. Consequently, search accuracy affects tightness only.
pub(crate) fn zcdp_log_delta(rho: f64, epsilon: f64) -> Fallible<f64> {
    check_rho(rho)?;
    check_epsilon(epsilon)?;

    if rho == 0.0 || epsilon.is_infinite() {
        return Ok(f64::NEG_INFINITY);
    }
    if rho.is_infinite() {
        return Ok(0.0);
    }

    // log(delta) is always at most zero. Starting from zero provides the
    // trivial delta = 1 fallback if a certified candidate cannot be evaluated.
    let mut best: f64 = 0.0;

    let branch0 = optimize_alpha_log_gap(log_gap_upper_delta0(rho, epsilon), |alpha| {
        search_log_delta0(alpha, rho, epsilon)
    });
    visit_certification_candidates(branch0, |alpha| {
        if let Some(value) = certify_log_delta0(alpha, rho, epsilon) {
            best = best.min(value);
        }
    });

    if let Some(log_gap_hi) = log_gap_upper_delta1(rho, epsilon) {
        let branch1 =
            optimize_alpha_log_gap(log_gap_hi, |alpha| search_log_delta1(alpha, rho, epsilon));
        visit_certification_candidates(branch1, |alpha| {
            if let Some(value) = certify_log_delta1(alpha, rho, epsilon) {
                best = best.min(value);
            }
        });
    }

    Ok(best.min(0.0))
}

#[proven(proof_path = "measures/zcdp/cdp_delta.tex")]
/// Return a conservative upper bound on delta for rho-zCDP at epsilon.
pub(crate) fn zcdp_delta(rho: f64, epsilon: f64) -> Fallible<f64> {
    log_to_delta_upper(zcdp_log_delta(rho, epsilon)?)
}

/// Return a conservative upper bound on epsilon for rho-zCDP at delta.
///
/// This directly optimizes the two fixed-order inverse formulas. It does not
/// invert [`zcdp_delta`], avoiding the previous 50--100 nested certified
/// forward evaluations.
pub(crate) fn zcdp_epsilon(rho: f64, delta: f64) -> Fallible<f64> {
    check_rho(rho)?;
    check_delta(delta)?;

    if rho == 0.0 || delta == 1.0 {
        return Ok(0.0);
    }
    if rho.is_infinite() || delta == 0.0 {
        return Ok(f64::INFINITY);
    }

    // At epsilon = 0, the forward guarantee already covers this target.
    // This also preserves the exact zero boundary instead of returning a tiny
    // positive artifact from the fixed-order inverse formulas.
    if delta >= zcdp_delta(rho, 0.0)? {
        return Ok(0.0);
    }

    let mut best = f64::INFINITY;

    let branch0 = optimize_alpha_log_gap(log_gap_upper_epsilon0(rho, delta), |alpha| {
        search_epsilon0(alpha, rho, delta)
    });
    visit_certification_candidates(branch0, |alpha| {
        if let Some(value) = certify_epsilon0(alpha, rho, delta) {
            best = best.min(value);
        }
    });

    if let Some(log_gap_hi) = log_gap_upper_epsilon1(delta) {
        let branch1 =
            optimize_alpha_log_gap(log_gap_hi, |alpha| search_epsilon1(alpha, rho, delta));
        visit_certification_candidates(branch1, |alpha| {
            if let Some(value) = certify_epsilon1(alpha, rho, delta) {
                best = best.min(value);
            }
        });
    }

    Ok(best.max(0.0))
}

// -----------------------------------------------------------------------------
// Backend-generic zCDP adapters
// -----------------------------------------------------------------------------

fn zcdp_log_delta0_on<Bk>(alpha: f64, rho: f64, epsilon: f64) -> Fallible<Interval<Bk>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    let alpha = Interval::<Bk>::point(alpha)?;
    let gamma = alpha.clone().mul(Interval::<Bk>::point(rho)?)?;
    rdp_log_delta0_on(alpha, gamma, Interval::<Bk>::point(epsilon)?)
}

fn zcdp_log_delta1_on<Bk>(alpha: f64, rho: f64, epsilon: f64) -> Fallible<Option<Interval<Bk>>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    let alpha = Interval::<Bk>::point(alpha)?;
    let gamma = alpha.clone().mul(Interval::<Bk>::point(rho)?)?;
    rdp_log_delta1_on(alpha, gamma, Interval::<Bk>::point(epsilon)?)
}

fn zcdp_epsilon0_on<Bk>(alpha: f64, rho: f64, delta: f64) -> Fallible<Interval<Bk>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    let alpha = Interval::<Bk>::point(alpha)?;
    let gamma = alpha.clone().mul(Interval::<Bk>::point(rho)?)?;
    rdp_epsilon0_on(alpha, gamma, Interval::<Bk>::point(delta)?)
}

fn zcdp_epsilon1_on<Bk>(alpha: f64, rho: f64, delta: f64) -> Fallible<Option<Interval<Bk>>>
where
    Bk: IntervalArithmeticBackend + IntervalExpBackend,
{
    let alpha = Interval::<Bk>::point(alpha)?;
    let gamma = alpha.clone().mul(Interval::<Bk>::point(rho)?)?;
    rdp_epsilon1_on(alpha, gamma, Interval::<Bk>::point(delta)?)
}

// -----------------------------------------------------------------------------
// Search objectives: approximate native interval arithmetic
// -----------------------------------------------------------------------------

fn search_log_delta0(alpha: f64, rho: f64, epsilon: f64) -> f64 {
    zcdp_log_delta0_on::<A>(alpha, rho, epsilon)
        .and_then(|value| value.upper_f64())
        .map(clean_log_delta_search_value)
        .unwrap_or(f64::INFINITY)
}

fn search_log_delta1(alpha: f64, rho: f64, epsilon: f64) -> f64 {
    zcdp_log_delta1_on::<A>(alpha, rho, epsilon)
        .and_then(|value| value.map(|value| value.upper_f64()).transpose())
        .ok()
        .flatten()
        .map(clean_log_delta_search_value)
        .unwrap_or(f64::INFINITY)
}

fn search_epsilon0(alpha: f64, rho: f64, delta: f64) -> f64 {
    zcdp_epsilon0_on::<A>(alpha, rho, delta)
        .and_then(|value| value.upper_f64())
        .map(clean_epsilon_search_value)
        .unwrap_or(f64::INFINITY)
}

fn search_epsilon1(alpha: f64, rho: f64, delta: f64) -> f64 {
    zcdp_epsilon1_on::<A>(alpha, rho, delta)
        .and_then(|value| value.map(|value| value.upper_f64()).transpose())
        .ok()
        .flatten()
        .map(clean_epsilon_search_value)
        .unwrap_or(f64::INFINITY)
}

// -----------------------------------------------------------------------------
// Final certification: soft directed interval arithmetic
// -----------------------------------------------------------------------------

fn certify_log_delta0(alpha: f64, rho: f64, epsilon: f64) -> Option<f64> {
    zcdp_log_delta0_on::<D>(alpha, rho, epsilon)
        .and_then(|value| value.upper_f64())
        .ok()
        .and_then(clean_certified_log_delta)
}

fn certify_log_delta1(alpha: f64, rho: f64, epsilon: f64) -> Option<f64> {
    zcdp_log_delta1_on::<D>(alpha, rho, epsilon)
        .ok()
        .flatten()
        .and_then(|value| value.upper_f64().ok())
        .and_then(clean_certified_log_delta)
}

fn certify_epsilon0(alpha: f64, rho: f64, delta: f64) -> Option<f64> {
    zcdp_epsilon0_on::<D>(alpha, rho, delta)
        .and_then(|value| value.upper_f64())
        .ok()
        .and_then(clean_certified_epsilon)
}

fn certify_epsilon1(alpha: f64, rho: f64, delta: f64) -> Option<f64> {
    zcdp_epsilon1_on::<D>(alpha, rho, delta)
        .ok()
        .flatten()
        .and_then(|value| value.upper_f64().ok())
        .and_then(clean_certified_epsilon)
}

// -----------------------------------------------------------------------------
// Renyi-order search in u = log(alpha - 1)
// -----------------------------------------------------------------------------

fn optimize_alpha_log_gap(log_gap_hi: f64, objective: impl Fn(f64) -> f64) -> BracketedOptimum {
    let log_gap_lo = log_gap_min();
    let log_gap_hi = log_gap_hi.clamp(log_gap_lo, log_gap_hard_cap());

    optimize_to_precision_bracket(
        SearchMode::Minimize,
        log_gap_lo,
        log_gap_hi,
        None,
        |log_gap| objective(alpha_from_log_gap(log_gap)),
    )
}

fn visit_certification_candidates(optimum: BracketedOptimum, mut visit: impl FnMut(f64)) {
    let mut previous = None;
    for log_gap in [optimum.lo, optimum.arg, optimum.hi] {
        let alpha = alpha_from_log_gap(log_gap);
        if previous == Some(alpha) {
            continue;
        }
        previous = Some(alpha);
        visit(alpha);
    }
}

fn alpha_from_log_gap(log_gap: f64) -> f64 {
    let alpha = 1.0 + log_gap.exp();
    if alpha > 1.0 {
        alpha.min(ALPHA_HARD_CAP)
    } else {
        1.0f64.next_up()
    }
}

fn log_gap_min() -> f64 {
    // The smallest representable alpha > 1 has alpha - 1 = f64::EPSILON.
    f64::EPSILON.ln()
}

fn log_gap_hard_cap() -> f64 {
    (ALPHA_HARD_CAP - 1.0).ln()
}

/// Branch-zero forward-search cap inherited from the CKS analysis.
fn log_gap_upper_delta0(rho: f64, epsilon: f64) -> f64 {
    let alpha_hi = (epsilon + 1.0) / (2.0 * rho) + 2.0;
    let alpha_hi = if alpha_hi.is_finite() && alpha_hi > 1.0 {
        alpha_hi.min(ALPHA_HARD_CAP)
    } else {
        ALPHA_HARD_CAP
    };

    (alpha_hi - 1.0).ln()
}

/// Branch-one forward domain: alpha < epsilon / rho, equivalently
/// alpha - 1 < (epsilon - rho) / rho.
fn log_gap_upper_delta1(rho: f64, epsilon: f64) -> Option<f64> {
    if epsilon <= rho {
        return None;
    }

    let domain = (epsilon - rho).ln() - rho.ln();
    strict_log_gap_upper(domain)
}

/// Find a branch-zero inverse-search cap by locating a point where the
/// derivative is nonnegative. This search is heuristic-only and therefore uses
/// ordinary floating point.
fn log_gap_upper_epsilon0(rho: f64, delta: f64) -> f64 {
    let mut log_gap = 0.0; // alpha = 2
    if epsilon0_derivative_nonnegative(log_gap, rho, delta) {
        return log_gap;
    }

    let hard_cap = log_gap_hard_cap();
    while log_gap < hard_cap {
        let next = (log_gap + std::f64::consts::LN_2).min(hard_cap);
        if next == log_gap {
            break;
        }
        log_gap = next;

        if epsilon0_derivative_nonnegative(log_gap, rho, delta) {
            break;
        }
    }

    log_gap
}

fn epsilon0_derivative_nonnegative(log_gap: f64, rho: f64, delta: f64) -> bool {
    let alpha = alpha_from_log_gap(log_gap);
    let log_alpha_delta = alpha.ln() + delta.ln();

    // d epsilon_0 / d alpha
    //   = rho + log(alpha * delta) / (alpha - 1)^2.
    if log_alpha_delta >= 0.0 {
        return true;
    }

    // Compare
    //
    //     rho * (alpha - 1)^2 >= -log(alpha * delta)
    //
    // in log space to avoid overflow and underflow.
    rho.ln() + 2.0 * log_gap >= (-log_alpha_delta).ln()
}

/// Branch-one inverse domain: alpha * delta < 1, equivalently
/// alpha - 1 < (1 - delta) / delta.
fn log_gap_upper_epsilon1(delta: f64) -> Option<f64> {
    let domain = (-delta).ln_1p() - delta.ln();
    strict_log_gap_upper(domain)
}

fn strict_log_gap_upper(domain: f64) -> Option<f64> {
    if domain.is_nan() || domain <= log_gap_min() {
        return None;
    }

    let hard_cap = log_gap_hard_cap();
    let upper = if domain <= hard_cap {
        // The theorem condition is strict. Move the search endpoint into the
        // interior; the fixed-order certified kernel independently rechecks it.
        domain.next_down()
    } else {
        hard_cap
    };

    (upper >= log_gap_min()).then_some(upper)
}

// -----------------------------------------------------------------------------
// Result and input handling
// -----------------------------------------------------------------------------

fn clean_log_delta_search_value(value: f64) -> f64 {
    if value.is_nan() {
        f64::INFINITY
    } else {
        value.min(0.0)
    }
}

fn clean_epsilon_search_value(value: f64) -> f64 {
    if value.is_nan() || value < 0.0 {
        f64::INFINITY
    } else {
        value.max(0.0)
    }
}

fn clean_certified_log_delta(value: f64) -> Option<f64> {
    (!value.is_nan()).then_some(value.min(0.0))
}

fn clean_certified_epsilon(value: f64) -> Option<f64> {
    (!value.is_nan() && value >= 0.0).then_some(value.max(0.0))
}

fn log_to_delta_upper(log_delta: f64) -> Fallible<f64> {
    if log_delta.is_nan() {
        return fallible!(FailedMap, "computed log(delta) is NaN");
    }
    if log_delta == f64::NEG_INFINITY {
        return Ok(0.0);
    }
    if log_delta >= 0.0 {
        return Ok(1.0);
    }

    // Avoid asking the arbitrary-precision backend to exponentiate beyond its
    // representable exponent range. The smallest subnormal is conservative for
    // every smaller positive result.
    let min_positive = f64::from_bits(1);
    if log_delta <= min_positive.ln() {
        return Ok(min_positive);
    }

    // InfExp rounds upward and returns the smallest positive f64 on underflow.
    Ok(log_delta.inf_exp()?.min(1.0))
}

fn check_rho(rho: f64) -> Fallible<()> {
    if rho.is_nan() {
        return fallible!(FailedMap, "rho must not be NaN");
    }
    if rho.is_sign_negative() {
        return fallible!(FailedMap, "rho ({rho}) must be non-negative");
    }
    Ok(())
}

fn check_epsilon(epsilon: f64) -> Fallible<()> {
    if epsilon.is_nan() {
        return fallible!(FailedMap, "epsilon must not be NaN");
    }
    if epsilon.is_sign_negative() {
        return fallible!(FailedMap, "epsilon ({epsilon}) must be non-negative");
    }
    Ok(())
}

fn check_delta(delta: f64) -> Fallible<()> {
    if delta.is_nan() {
        return fallible!(FailedMap, "delta must not be NaN");
    }
    if delta.is_sign_negative() || delta > 1.0 {
        return fallible!(FailedMap, "delta ({delta}) must be between zero and one");
    }
    Ok(())
}
