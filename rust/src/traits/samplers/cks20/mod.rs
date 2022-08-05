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
//     https://github.com/IBM/discrete-gaussian-differential-privacy/blob/cb190d2a990a78eff6e21159203bc888e095f01b/discretegauss.py

use rug::{Rational, Integer};
use crate::traits::samplers::SampleBernoulli;
use crate::error::Fallible;

use num::{Zero, One};

use super::{SampleUniformIntBelow, SampleRademacher};


// sample from a Bernoulli(exp(-x)) distribution
// assumes x is a rational number in [0,1]
fn sample_bernoulli_exp1(x: Rational) -> Fallible<bool> {
    let mut k = Integer::one();
    loop {
        if bool::sample_bernoulli(x.clone() / &k, false)? {
            k += 1;
        } else {
            return Ok(k.is_odd());
        }
    }
}

// sample from a Bernoulli(exp(-x)) distribution
// assumes x is a rational number >=0
fn sample_bernoulli_exp(mut x: Rational) -> Fallible<bool> {
    // Sample floor(x) independent Bernoulli(exp(-1))
    // If all are 1, return Bernoulli(exp(-(x-floor(x))))
    while x > 1 {
        if sample_bernoulli_exp1(1.into())? {
            x -= 1;
        } else {
            return Ok(false);
        }
    }
    sample_bernoulli_exp1(x)
}

// sample from a geometric(1-exp(-x)) distribution
// assumes x is a rational number >= 0
fn sample_geometric_exp_slow(x: Rational) -> Fallible<Integer> {
    let mut k = 0.into();
    loop {
        if sample_bernoulli_exp(x.clone())? {
            k += 1;
        } else {
            return Ok(k);
        }
    }
}

// sample from a geometric(1-exp(-x)) distribution
// assumes x >= 0 rational
fn sample_geometric_exp_fast(x: Rational) -> Fallible<Integer> {
    if x.is_zero() {
        return Ok(0.into());
    }

    let (numer, denom) = x.into_numer_denom();
    let mut u = Integer::sample_uniform_int_below(denom.clone())?;
    while !sample_bernoulli_exp(Rational::from((u.clone(), denom.clone())))? {
        u = Integer::sample_uniform_int_below(denom.clone())?;
    }
    let v2 = sample_geometric_exp_slow(Rational::one())?;
    Ok((v2 * denom + u) / numer)
}

pub fn sample_discrete_laplace(scale: Rational) -> Fallible<Integer> {
    if scale.is_zero() {
        return Ok(0.into())
    }
    loop {
        let sign = Integer::sample_standard_rademacher()?;
        let magnitude = sample_geometric_exp_fast(scale.clone().recip())?;
        if !(sign.is_one() && magnitude.is_zero()) {
            return Ok(sign * magnitude);
        }
    }
}


pub fn sample_discrete_gaussian(scale: Rational) -> Fallible<Integer> {
    let t = scale.clone().floor() + 1i8;
    let sigma2 = scale.square();
    loop {
        let candidate = sample_discrete_laplace(t.clone())?;
        let x = candidate.clone().abs() - sigma2.clone() / &t;
        let bias = x.square() / (2 * sigma2.clone());
        if sample_bernoulli_exp(bias)? {
            return Ok(candidate);
        }
    }
}
