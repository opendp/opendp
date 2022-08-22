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
//     https://github.com/IBM/discrete-gaussian-differential-privacy/blob/cb190d2a990a78eff6e21159203bc888e095f01b/cdp2adp.py

use crate::{error::Fallible, traits::Float};

pub(crate) fn cdp_epsilon<Q: Float>(rho: Q, delta: Q) -> Fallible<Q> {
    if rho.is_sign_negative() {
        return fallible!(FailedRelation, "rho must be non-negative");
    }

    if delta.is_sign_negative() {
        return fallible!(FailedRelation, "delta must be non-negative");
    }

    if rho.is_zero() {
        return Ok(Q::zero());
    }

    let _1 = Q::one();
    let _2 = _1 + _1;

    // It has been proven that...
    //     delta = exp((α-1) (αρ - ε) + α ln1p(-1/α)) / (α-1)
    // ...for any choice of alpha in (1, infty)
    
    // The following expression is equivalent for ε:
    //   epsilon = δρ + (ln(1/δ) + (α - 1)ln(1 - 1/α) - ln(α)) / (α - 1)

    // This algorithm searches for the best alpha, the alpha that minimizes delta.

    // Since any alpha in (1, infty) yields a valid upper bound on epsilon,
    //    the search for alpha does not need conservative rounding.
    // If this search is slightly "incorrect" by float rounding it will only result in larger delta (still valid)

    // We now choose bounds for the binary search over alpha.

    // Take the derivative wrt α and check if positive:
    let deriv_pos = |a: Q| rho > -(a * delta).ln() / (a - _1).powi(2);
    //                     ρ   > -ln(αδ)           / (α - 1)^2

    // Don't let alpha be too small, due to numerical stability.
    // We only encounter α <= 1.01 when eps <= rho or close to it.
    // This is not an interesting parameter regime, as you will
    //     inherently get large delta in this regime.
    let mut a_min = Q::round_cast(1.01f64)?;

    // Find an upper bound for alpha via an exponential search
    let mut a_max = _2;
    while !deriv_pos(a_max) {
        a_max *= _2;
    }

    // run binary search to find ideal alpha
    // Since the function is convex (when restricted to the bounds)
    //     the ideal alpha is the critical point of the derivative of the function for delta
    loop {
        let diff = a_max - a_min;

        let a_mid = a_min + diff / _2;

        if a_mid == a_max || a_mid == a_min {
            break;
        }

        if deriv_pos(a_mid) {
            a_max = a_mid;
        } else {
            a_min = a_mid;
        }
    }

    // calculate epsilon

    let a_m1 = a_max.inf_sub(&_1)?;

    // reorganize 1 - 1/α -> (α-1)/α for numerical stability
    //  numer = ln(1/δ) + (α-1) ln((α-1)/α) - ln(α)
    let numer = (a_m1.inf_div(&a_max)?.inf_ln()?.inf_mul(&a_m1)?)
        .inf_sub(&a_max.inf_ln()?)?
        .inf_add(&delta.recip().inf_ln()?)?;

    //  denom = α - 1
    let denom = a_max.neg_inf_sub(&_1)?;

    //  epsilon = δρ + (ln(1/δ) + (α-1) ln((α-1)/α) - ln(α)) / (α - 1)
    //                  -----------------------------------  /  -----
    //          = δρ                          + numer        / denom
    let epsilon = a_max.inf_mul(&rho)?.inf_add(&numer.inf_div(&denom)?)?;

    Ok(epsilon)
}
