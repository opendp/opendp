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
//     https://github.com/IBM/discrete-gaussian-differential-privacy/blob/cb190d2a990a78eff6e21159203bc888e095f01b/discretegauss.py

use crate::error::Fallible;
use dashu::{
    base::Abs,
    integer::{IBig, UBig},
    rational::RBig,
    rbig,
};
use opendp_derive::proven;

use super::{sample_bernoulli_rational, sample_standard_bernoulli, sample_uniform_ubig_below};

#[proven]
/// Sample exactly from the Bernoulli(exp(-x)) distribution, where $x \in [0, 1]$.
///
/// # Proof Definition
/// For any rational number in [0, 1], `x`,
/// `sample_bernoulli_exp1` either returns `Err(e)`, due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Bernoulli(exp(-x))$.
fn sample_bernoulli_exp1(x: RBig) -> Fallible<bool> {
    let mut k = UBig::ONE;
    loop {
        if sample_bernoulli_rational(x.clone() / &k)? {
            k += UBig::ONE;
        } else {
            return Ok(k % 2u8 == 1);
        }
    }
}

#[proven]
/// Sample exactly from the Bernoulli(exp(-x)) distribution, where $x \ge 0$.
///
/// # Proof Definition
/// For any non-negative finite rational `x`,
/// `sample_bernoulli_exp` either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Bernoulli(exp(-x))$.

fn sample_bernoulli_exp(mut x: RBig) -> Fallible<bool> {
    // Sample floor(x) independent Bernoulli(exp(-1))
    // If all are 1, return Bernoulli(exp(-(x-floor(x))))
    while x > RBig::ONE {
        if sample_bernoulli_exp1(1.into())? {
            x -= RBig::ONE;
        } else {
            return Ok(false);
        }
    }
    sample_bernoulli_exp1(x)
}

#[proven]
/// Sample exactly from the geometric distribution (slow).
///
/// # Proof Definition
/// For any non-negative rational `x`,
/// `sample_geometric_exp_slow` either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Geometric(1 - exp(-x))$.
fn sample_geometric_exp_slow(x: RBig) -> Fallible<UBig> {
    let mut k = UBig::ZERO;
    loop {
        if sample_bernoulli_exp(x.clone())? {
            k += UBig::ONE;
        } else {
            return Ok(k);
        }
    }
}

#[proven]
/// Sample exactly from the geometric distribution (fast).
///
/// # Proof Definition
/// For any non-negative rational `x`,
/// `sample_geometric_exp_fast` either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Geometric(1 - exp(-x))$.
pub(crate) fn sample_geometric_exp_fast(x: RBig) -> Fallible<UBig> {
    if x.is_zero() {
        return Ok(UBig::ZERO);
    }

    let (numer, denom) = x.into_parts();
    let mut u = sample_uniform_ubig_below(denom.clone())?;
    while !sample_bernoulli_exp(RBig::from_parts(u.as_ibig().clone(), denom.clone()))? {
        u = sample_uniform_ubig_below(denom.clone())?;
    }
    let v2 = sample_geometric_exp_slow(RBig::ONE)?;
    Ok((v2 * denom + u) / numer.into_parts().1)
}

#[proven]
/// Sample exactly from the discrete laplace distribution with arbitrary precision.
///
/// # Proof Definition
/// For any `scale` that is a non-negative rational number,
/// `sample_discrete_laplace` either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $\mathcal{L}_\mathbb{Z}(0, scale)$.
///
/// Specifically, the probability of returning any `x` of type [`IBig`] is
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{e^{-1/scale} - 1}{e^{-1/scale} + 1} e^{-|x|/scale}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{Z}(0, scale)
/// ```
///
/// # Citation
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/abs/2004.00010)
pub fn sample_discrete_laplace(scale: RBig) -> Fallible<IBig> {
    if scale.is_zero() {
        return Ok(0.into());
    }
    let (numer, denom) = scale.into_parts();
    let inv_scale = RBig::from_parts(denom.as_ibig().clone(), numer.into_parts().1);

    loop {
        let positive = sample_standard_bernoulli()?;
        let magnitude = sample_geometric_exp_fast(inv_scale.clone())?
            .as_ibig()
            .clone();
        if positive || !magnitude.is_zero() {
            return Ok(if positive { magnitude } else { -magnitude });
        }
    }
}

#[proven]
/// Sample exactly from the discrete gaussian distribution with arbitrary precision.
/// # Proof Definition
/// For any `scale` that is a non-negative rational number,
/// `sample_discrete_gaussian` either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $\mathcal{N}_\mathbb{Z}(0, scale^2)$.
///
/// Specifically, the probability of returning any `x` of type [`IBig`] is
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{e^{-\frac{x^2}{2\sigma^2}}}{\sum_{y\in\mathbb{Z}}e^{-\frac{y^2}{2\sigma^2}}}, \quad
/// \text{where } X \sim \mathcal{N}_\mathbb{Z}(0, \sigma^2)
/// ```
/// where $\sigma = scale$.
///
/// # Citation
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/abs/2004.00010)
pub fn sample_discrete_gaussian(scale: RBig) -> Fallible<IBig> {
    if scale.is_zero() {
        return Ok(IBig::ZERO);
    }
    let t = RBig::from(scale.clone().floor() + 1i8);
    let sigma2 = scale.pow(2);
    loop {
        let candidate = sample_discrete_laplace(t.clone())?;
        let x = (&candidate).abs() - sigma2.clone() / &t;
        let bias = x.pow(2) / (sigma2.clone() * rbig!(2));
        if sample_bernoulli_exp(bias)? {
            return Ok(candidate);
        }
    }
}
