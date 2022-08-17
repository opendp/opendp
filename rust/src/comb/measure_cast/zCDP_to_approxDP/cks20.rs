// Copyright 2022 OpenDP
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Algorithm from:
//     Clément Canonne, Gautam Kamath, Thomas Steinke. The Discrete Gaussian for Differential Privacy. 2020.
//     https://arxiv.org/abs/2004.00010
//
// This file is derived from the following implementation by Thomas Steinke:
//     https://github.com/IBM/discrete-gaussian-differential-privacy/blob/cb190d2a990a78eff6e21159203bc888e095f01b/cdp2adp.py

use crate::{error::Fallible, traits::Float};

pub(crate) fn cdp_epsilon<Q: Float>(rho: Q, delta: Q) -> Fallible<Q> {
    let tol = Q::round_cast(1e-8)?;
    let _2 = Q::one() + Q::one();

    if delta.is_sign_negative() {
        return fallible!(FailedRelation, "delta must be non-negative");
    }

    if delta >= Q::one() || rho.is_zero() {
        return Ok(Q::zero());
    }

    // conduct a binary search for epsilon (e_max)

    // epsilon is non-negative. Maintain that cdp_delta(rho, e_min) >= delta
    let mut e_min = Q::zero();

    // use the standard bound as e_max. Maintain cdp_delta(rho, e_max) <= delta
    // e_max = 2 sqrt(ln(1/δ)ρ) + ρ
    let mut e_max = delta
        .recip()
        .inf_ln()?
        .inf_mul(&rho)?
        .inf_sqrt()?
        .inf_mul(&_2)?
        .inf_add(&rho)?;

    loop {
        let diff = e_max - e_min;
        if diff <= tol {
            break;
        }

        let e_mid = e_min + diff / _2;
        if cdp_delta(rho, e_mid)? <= delta {
            e_max = e_mid;
        } else {
            e_min = e_mid;
        }
    }
    Ok(e_max)
}

fn cdp_delta<Q>(rho: Q, eps: Q) -> Fallible<Q>
where
    Q: Float,
{
    if rho.is_sign_negative() {
        return fallible!(FailedRelation, "rho must be non-negative");
    }

    if eps.is_sign_negative() {
        return fallible!(FailedRelation, "epsilon must be non-negative");
    }

    if rho.is_zero() {
        return Ok(Q::zero());
    }

    let tol = Q::round_cast(1e-8)?;
    let _1 = Q::one();
    let _2 = _1 + _1;

    // search for best alpha
    // Any alpha in (1, infty) yields a valid upper bound on delta.
    // Therefore the search does not need to be exact
    // Thus if this search is slightly "incorrect" it will only result in larger delta (still valid)

    // Don't let alpha be too small, due to numerical stability.
    // The optimal alpha is at least (1+eps/rho)/2,
    //     thus we only encounter α <= 1.01 when eps <= rho or close to it.
    // This is not an interesting parameter regime, as you will
    //     inherently get large delta in this regime.
    let mut a_min = Q::round_cast(1.01f64)?;

    // (ε+1)/(2ρ) + 2
    let mut a_max = eps
        .inf_add(&_1)?
        .inf_div(&_2.neg_inf_mul(&rho)?)?
        .inf_add(&_2)?;

    loop {
        let diff = a_max - a_min;
        if diff <= tol {
            break;
        }

        let a_mid = a_min + diff / _2;

        // calculate derivative
        let deriv = (_2 * a_mid - _1) * rho - eps + a_mid.recip().neg().ln_1p();
        //        = (2α - 1)            ρ   - ε   + ln1p(-1/α)

        if deriv.is_sign_negative() {
            a_min = a_mid;
        } else {
            a_max = a_mid;
        }
    }

    // calculate delta
    //  t1 = (α-1) (αρ - ε)
    let t1 = a_max
        .inf_sub(&_1)?
        .inf_mul(&(a_max.inf_mul(&rho)?.inf_sub(&eps)?))?;

    //  t2 = α ln1p(-1/α)
    let t2 = a_max.inf_mul(&a_max.recip().neg().inf_ln_1p()?)?;

    //  delta = exp((α-1) (αρ - ε) + α ln1p(-1/α)) / (α-1)
    //        = exp(t1             + t2          ) / (α-1)
    let delta = t1
        .inf_add(&t2)?
        .inf_exp()?
        .inf_div(&(a_max.inf_sub(&_1)?))?;

    // delta is always <= 1
    Ok(delta.min(Q::one()))
}
