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

use std::ops::Neg;

use num::Zero;

use crate::{
    error::Fallible,
    traits::{InfAdd, InfDiv, InfExp, InfLn1P, InfMul, InfSub},
};

#[cfg(test)]
pub(crate) mod test;

/// # Proof Definition
///
/// For any possible setting of $\rho$ and $\epsilon$, $\texttt{cdp\_delta}$ either returns an error,
/// or a $\delta$ such that any $\rho$-differentially private measurement is also $(\epsilon, \delta)$-differentially private.
pub(crate) fn cdp_delta(rho: f64, eps: f64) -> Fallible<f64> {
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

    // It has been proven that...
    //    delta = exp((α-1) (αρ - ε) + α ln1p(-1/α)) / (α-1)
    // ...for any choice of alpha in (1, infty)

    // This algorithm searches for the best alpha, the alpha that minimizes delta.

    // Since any alpha in (1, infty) yields a valid upper bound on delta,
    //    the search for alpha does not need conservative rounding.
    // If this search is slightly "incorrect" by float rounding it will only result in larger delta (still valid)

    // We now choose bounds for the binary search over alpha.

    // The optimal alpha is no greater than (ε+1)/(2ρ)
    let mut a_max = eps
        .inf_add(&1.0)?
        .inf_div(&(2.0).neg_inf_mul(&rho)?)?
        .inf_add(&2.0)?;

    // Don't let alpha be too small, due to numerical stability.
    // We only encounter α <= 1.01 when eps <= rho or close to it.
    // This is not an interesting parameter regime, as you will
    //     inherently get large delta in this regime.
    let mut a_min = 1.01f64;

    // run binary search to find ideal alpha
    // Since the function is convex (when restricted to the bounds)
    //     the ideal alpha is the critical point of the derivative of the function for delta
    loop {
        let diff = a_max - a_min;

        let a_mid = a_min + diff / 2.0;

        if a_mid == a_max || a_mid == a_min {
            break;
        }

        // calculate derivative
        let deriv = (2.0 * a_mid - 1.0) * rho - eps + a_mid.recip().neg().ln_1p();
        //        = (2α - 1)            * ρ   - ε   + ln1p(-1/α)

        if deriv.is_sign_negative() {
            a_min = a_mid;
        } else {
            a_max = a_mid;
        }
    }

    // calculate delta
    let a_1 = a_max.inf_sub(&1.0)?;
    let ar_e = a_max.inf_mul(&rho)?.inf_sub(&eps)?;
    //  t1 = (α-1) (αρ - ε)
    let t1 = match a_1.inf_mul(&ar_e) {
        // if t1 is negative, then handle negative overflow by making t1 larger: the most negative finite float
        // making t1 larger makes delta larger, so it's still a valid upper bound
        Err(_) if a_1.is_sign_negative() != ar_e.is_sign_negative() => f64::MIN,
        Ok(v) => v,
        err => err?,
    };

    //  t2 = α ln1p(-1/α)
    let t2 = a_max.inf_mul(&a_max.recip().neg().inf_ln_1p()?)?;

    //  delta = exp((α-1) (αρ - ε) + α ln1p(-1/α)) / (α-1)
    //        = exp(t1             + t2          ) / (α-1)
    let delta = t1
        .inf_add(&t2)?
        .inf_exp()?
        .inf_div(&(a_max.inf_sub(&1.0)?))?;

    // delta is always <= 1
    Ok(delta.min(1.0))
}
