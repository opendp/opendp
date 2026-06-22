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

// Algorithm from:
//     Clément Canonne, Gautam Kamath, Thomas Steinke. The Discrete Gaussian for Differential Privacy. 2020.
//     https://arxiv.org/abs/2004.00010
//
// This file is derived from the following implementation by Thomas Steinke:
//     https://github.com/IBM/discrete-gaussian-differential-privacy/blob/cb190d2a990a78eff6e21159203bc888e095f01b/cdp2adp.py#L74-L102

use num::Zero;
use opendp_derive::proven;

use crate::{error::Fallible, traits::InfExp};

#[cfg(test)]
pub(crate) mod test;

#[proven]
/// # Proof Definition
///
/// For any possible setting of $\rho$ and $\epsilon$, $\texttt{cdp\_delta}$ either returns an error,
/// or a $\delta$ such that any $\rho$-differentially private measurement is also $(\epsilon, \delta)$-differentially private.
pub(crate) fn cdp_delta(rho: f64, eps: f64) -> Fallible<f64> {
    if rho.is_nan() {
        return fallible!(FailedMap, "rho must not be NaN");
    }

    if eps.is_nan() {
        return fallible!(FailedMap, "epsilon must not be NaN");
    }

    if rho.is_sign_negative() {
        return fallible!(FailedMap, "rho ({}) must be non-negative", rho);
    }

    if eps.is_sign_negative() {
        return fallible!(FailedMap, "epsilon ({}) must be non-negative", eps);
    }

    if rho.is_zero() || eps.is_infinite() {
        return Ok(0.0);
    }

    if rho.is_infinite() {
        return Ok(1.0);
    }

    // We search over alpha using ordinary floating point.
    // This is fine: any alpha > 1 gives a valid bound.
    // The search only affects tightness, not validity, provided final evaluation is conservative enough.

    let mut best_log_delta = f64::INFINITY;

    // ---------------------------------------------------------------------
    // Branch 0:
    //
    // delta_0(alpha)
    //   = exp((alpha - 1) * (alpha * rho - eps)
    //         + alpha * ln1p(-1 / alpha))
    //     / (alpha - 1)
    //
    // This is the bound you are already using.
    // ---------------------------------------------------------------------

    let alpha_lo = alpha_lower();

    let alpha0_hi = alpha0_upper(rho, eps);

    if alpha0_hi > alpha_lo {
        let alpha0 =
            minimize_unimodal_log(alpha_lo, alpha0_hi, |alpha| log_delta0(alpha, rho, eps));

        best_log_delta = best_log_delta.min(log_delta0(alpha0, rho, eps));
        best_log_delta = best_log_delta.min(log_delta0(alpha_lo, rho, eps));
        best_log_delta = best_log_delta.min(log_delta0(alpha0_hi, rho, eps));
    }

    // ---------------------------------------------------------------------
    // Branch 1:
    //
    // delta_1(alpha)
    //   = (exp((alpha - 1) * alpha * rho) - 1)
    //     / (alpha * (exp((alpha - 1) * eps) - 1))
    //
    // This branch is useful when eps > alpha * rho.
    // Therefore alpha must lie in:
    //
    //     1 < alpha < eps / rho
    //
    // ---------------------------------------------------------------------

    if eps > rho {
        if let Some((alpha1_lo, alpha1_hi)) = alpha1_interval(rho, eps) {
            if alpha1_hi > alpha1_lo {
                let alpha1 = minimize_unimodal_log(alpha1_lo, alpha1_hi, |alpha| {
                    log_delta1(alpha, rho, eps)
                });

                best_log_delta = best_log_delta.min(log_delta1(alpha1, rho, eps));
                best_log_delta = best_log_delta.min(log_delta1(alpha1_lo, rho, eps));
                best_log_delta = best_log_delta.min(log_delta1(alpha1_hi, rho, eps));
            } else {
                best_log_delta = best_log_delta.min(log_delta1(alpha1_lo, rho, eps));
            }
        }
    }

    log_to_delta(best_log_delta)
}

fn alpha_lower() -> f64 {
    // Avoid alpha extremely close to 1, where cancellation is unpleasant.
    // This is much less restrictive than 1.01, so it keeps the near-alpha=1
    // improvement from branch 1.
    1.0 + f64::EPSILON.sqrt()
}

fn alpha0_upper(rho: f64, eps: f64) -> f64 {
    // Same upper-bound idea as your existing implementation:
    //
    //     alpha* <= roughly (eps + 1) / (2 rho) + 2
    //
    // If this overflows, cap the search. Any alpha remains valid; this only
    // affects tightness in extreme regimes.
    let hi = (eps + 1.0) / (2.0 * rho) + 2.0;

    if hi.is_finite() && hi > alpha_lower() {
        hi
    } else {
        // Large enough to cover all practical finite optima while reducing
        // alpha^2 overflow risk in the objective.
        f64::MAX.sqrt()
    }
}

fn alpha1_interval(rho: f64, eps: f64) -> Option<(f64, f64)> {
    if !(eps > rho) {
        return None;
    }

    // alpha < eps / rho.
    // Compute the upper endpoint in log-space so eps / rho can overflow safely.
    let log_hi = eps.ln() - rho.ln();

    if !(log_hi > 0.0) || log_hi.is_nan() {
        return None;
    }

    let cap = f64::MAX.sqrt();
    let cap_log = cap.ln();

    let raw_hi = if log_hi >= cap_log { cap } else { log_hi.exp() };

    if !(raw_hi > 1.0) || !raw_hi.is_finite() {
        return None;
    }

    let gap = raw_hi - 1.0;

    // Pick an interior lower endpoint.
    let lo = if gap > 4.0 * f64::EPSILON.sqrt() {
        alpha_lower()
    } else {
        1.0 + gap * 0.25
    };

    // Pick an interior upper endpoint.
    let hi = if raw_hi < 2.0 {
        1.0 + gap * 0.75
    } else if raw_hi == cap {
        cap
    } else {
        raw_hi * (1.0 - 1e-12)
    };

    if lo > 1.0 && hi > 1.0 && hi >= lo {
        Some((lo, hi))
    } else {
        None
    }
}

fn log_delta0(alpha: f64, rho: f64, eps: f64) -> f64 {
    if !(alpha > 1.0) {
        return f64::INFINITY;
    }

    let alpha_m1 = alpha - 1.0;

    if !(alpha_m1 > 0.0) {
        return f64::INFINITY;
    }

    let gamma = alpha * rho;

    let out = alpha_m1 * (gamma - eps) + alpha * (-1.0 / alpha).ln_1p() - alpha_m1.ln();

    clean_log_objective(out)
}

fn log_delta1(alpha: f64, rho: f64, eps: f64) -> f64 {
    if !(alpha > 1.0) || !(eps > 0.0) || !(rho > 0.0) {
        return f64::INFINITY;
    }

    // Branch 1 is only valid/useful when eps > alpha * rho.
    // Do this comparison in log-space to avoid overflow in alpha * rho.
    let log_alpha = alpha.ln();
    let log_rho = rho.ln();
    let log_eps = eps.ln();

    if log_alpha + log_rho >= log_eps {
        return f64::INFINITY;
    }

    let alpha_m1 = alpha - 1.0;

    if !(alpha_m1 > 0.0) {
        return f64::INFINITY;
    }

    let log_alpha_m1 = alpha_m1.ln();

    // x = (alpha - 1) * alpha * rho
    // y = (alpha - 1) * eps
    //
    // delta_1 = expm1(x) / (alpha * expm1(y))
    //
    // Work with ln(x) and ln(y) to avoid underflow when x and y are tiny.
    let log_x = log_alpha_m1 + log_alpha + log_rho;
    let log_y = log_alpha_m1 + log_eps;

    let out = log_expm1_from_log_arg(log_x) - log_alpha - log_expm1_from_log_arg(log_y);

    clean_log_objective(out)
}

fn clean_log_objective(value: f64) -> f64 {
    if value.is_nan() { f64::INFINITY } else { value }
}

fn log_expm1_from_log_arg(log_x: f64) -> f64 {
    // Returns ln(exp(x) - 1), given ln(x).
    //
    // If x is tiny, expm1(x) ~= x, so ln(expm1(x)) ~= ln(x).
    // This avoids forming x when x would underflow.

    if log_x.is_nan() {
        return f64::NAN;
    }

    if log_x == f64::NEG_INFINITY {
        return f64::NEG_INFINITY;
    }

    if log_x < -37.0 {
        return log_x;
    }

    let x = log_x.exp();

    if x.is_infinite() {
        return f64::INFINITY;
    }

    log_expm1_pos(x)
}

fn log_expm1_pos(x: f64) -> f64 {
    // Returns ln(exp(x) - 1), for x >= 0.

    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }

    if x == 0.0 {
        return f64::NEG_INFINITY;
    }

    if x < std::f64::consts::LN_2 {
        x.exp_m1().ln()
    } else {
        // ln(exp(x) - 1)
        //   = x + ln(1 - exp(-x))
        x + (-(-x).exp()).ln_1p()
    }
}

fn minimize_unimodal_log<F>(mut lo: f64, mut hi: f64, f: F) -> f64
where
    F: Fn(f64) -> f64,
{
    debug_assert!(lo > 1.0);
    debug_assert!(hi >= lo);

    if !(hi > lo) {
        return lo;
    }

    // Golden-section search.
    const INV_PHI: f64 = 0.618_033_988_749_894_9;
    const INV_PHI2: f64 = 0.381_966_011_250_105_1;

    let mut c = lo + INV_PHI2 * (hi - lo);
    let mut d = lo + INV_PHI * (hi - lo);

    let mut fc = f(c);
    let mut fd = f(d);

    for _ in 0..160 {
        let width = hi - lo;

        if width <= f64::EPSILON * (lo.abs() + hi.abs()).max(1.0) {
            break;
        }

        if fc <= fd {
            hi = d;
            d = c;
            fd = fc;
            c = lo + INV_PHI2 * (hi - lo);
            fc = f(c);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + INV_PHI * (hi - lo);
            fd = f(d);
        }
    }

    let mid = lo + (hi - lo) / 2.0;

    let candidates = [(lo, f(lo)), (c, fc), (d, fd), (hi, f(hi)), (mid, f(mid))];

    let mut best_alpha = mid;
    let mut best_value = f(mid);

    for (alpha, value) in candidates {
        if value < best_value {
            best_alpha = alpha;
            best_value = value;
        }
    }

    best_alpha
}

fn log_to_delta(log_delta: f64) -> Fallible<f64> {
    if log_delta.is_nan() {
        return fallible!(FailedMap, "computed log(delta) is NaN");
    }

    if log_delta == f64::INFINITY {
        return Ok(1.0);
    }

    if log_delta >= 0.0 {
        return Ok(1.0);
    }

    // Smallest positive subnormal is 2^-1074.
    const LN_MIN_POSITIVE_SUBNORMAL: f64 = -744.440_071_921_381_2;

    // Add a small upward slack because the search/objective uses ordinary
    // libm transcendental functions. This makes the returned delta slightly
    // larger, which is the safe direction.
    let slack = 64.0 * f64::EPSILON * log_delta.abs().max(1.0);
    let log_delta = log_delta + slack;

    if log_delta <= LN_MIN_POSITIVE_SUBNORMAL {
        // Returning 0.0 would not be conservative unless the true value is
        // exactly zero. Here rho > 0 and eps < infinity, so return the
        // smallest positive f64 instead.
        return Ok(f64::from_bits(1));
    }

    let delta = log_delta.inf_exp()?;

    Ok(delta.min(1.0))
}
