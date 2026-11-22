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
    base::{Abs, Sign},
    integer::{IBig, UBig},
    rational::RBig,
};
use opendp_derive::proven;

use super::{EntropySource, SamplerEntropy, SystemEntropy, sample_bernoulli_ratio_with};

#[cfg(test)]
mod test;

#[proven]
/// Sample exactly from the Bernoulli(exp(-x)) distribution, where $x \in [0, 1]$.
///
/// # Proof Definition
/// For any rational number in [0, 1], `x`,
/// `sample_bernoulli_exp1` either returns `Err(e)`, due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Bernoulli(exp(-x))$.
// Retain the proof-linked scalar entry point; buffered paths use the ratio core.
#[cfg_attr(not(test), allow(dead_code))]
fn sample_bernoulli_exp1(x: RBig) -> Fallible<bool> {
    sample_bernoulli_exp1_with(x, &mut SystemEntropy)
}

fn sample_bernoulli_exp1_with(x: RBig, entropy: &mut impl EntropySource) -> Fallible<bool> {
    let (numer, denom) = x.into_parts();
    let (Sign::Positive, numer) = numer.into_parts() else {
        return fallible!(FailedFunction, "x must be nonnegative");
    };
    sample_bernoulli_exp1_ratio(&numer, &denom, entropy)
}

fn sample_bernoulli_exp1_ratio(
    numer: &UBig,
    denom: &UBig,
    entropy: &mut impl EntropySource,
) -> Fallible<bool> {
    if denom.is_zero() || numer > denom {
        return fallible!(FailedFunction, "x must lie in [0, 1]");
    }
    if numer.is_zero() {
        return Ok(true);
    }
    // exp(-1) is a common primitive. Its continuation probabilities are 1/k,
    // even when the input was an unreduced wide ratio d/d.
    let (numer, denom) = if numer == denom {
        (&UBig::ONE, &UBig::ONE)
    } else {
        (numer, denom)
    };
    let mut k = UBig::ONE;
    loop {
        // a/(b*k) is the same probability as (a/b)/k, without a division or
        // gcd reduction of wide rational numbers on every continuation trial.
        if sample_bernoulli_ratio_with(numer, &(denom * &k), entropy)? {
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
pub(crate) fn sample_bernoulli_exp(x: RBig) -> Fallible<bool> {
    sample_bernoulli_exp_with(x, &mut SystemEntropy)
}

fn sample_bernoulli_exp_with(x: RBig, entropy: &mut impl EntropySource) -> Fallible<bool> {
    let (numer, denom) = x.into_parts();
    let (Sign::Positive, numer) = numer.into_parts() else {
        return fallible!(FailedFunction, "x must be nonnegative");
    };
    sample_bernoulli_exp_ratio(numer, &denom, entropy)
}

fn sample_bernoulli_exp_ratio(
    mut numer: UBig,
    denom: &UBig,
    entropy: &mut impl EntropySource,
) -> Fallible<bool> {
    if denom.is_zero() {
        return fallible!(FailedFunction, "denominator must be positive");
    }
    // Factor out integer units, stopping at the first failed exp(-1) coin.
    // The residual ratio remains exact without gcd reduction.
    while &numer > denom {
        if !sample_bernoulli_exp1_ratio(&UBig::ONE, &UBig::ONE, entropy)? {
            return Ok(false);
        }
        numer -= denom;
    }
    sample_bernoulli_exp1_ratio(&numer, denom, entropy)
}

#[proven]
/// Sample exactly from the geometric distribution (slow).
///
/// # Proof Definition
/// For any non-negative rational `x`,
/// `sample_geometric_exp_slow` either returns `Err(e)` due to a lack of system entropy,
/// or `Ok(out)`, where `out` is distributed as $Geometric(1 - exp(-x))$.
// Retain the proof-linked scalar entry point for callers and tests.
#[cfg_attr(not(test), allow(dead_code))]
fn sample_geometric_exp_slow(x: RBig) -> Fallible<UBig> {
    sample_geometric_exp_slow_with(x, &mut SystemEntropy)
}

fn sample_geometric_exp_slow_with(x: RBig, entropy: &mut impl EntropySource) -> Fallible<UBig> {
    let mut k = UBig::ZERO;
    loop {
        if sample_bernoulli_exp_with(x.clone(), entropy)? {
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
    sample_geometric_exp_fast_with(x, &mut SystemEntropy)
}

fn sample_geometric_exp_fast_with(x: RBig, entropy: &mut impl EntropySource) -> Fallible<UBig> {
    if x.is_zero() {
        return Ok(UBig::ZERO);
    }

    let (numer, denom) = x.into_parts();
    let mut u = entropy.uniform_ubig_below(denom.clone())?;
    // u/denom is already in [0,1). Keep it unreduced for the exponential coin.
    while !sample_bernoulli_exp1_ratio(&u, &denom, entropy)? {
        u = entropy.uniform_ubig_below(denom.clone())?;
    }
    let v2 = sample_geometric_exp_slow_with(RBig::ONE, entropy)?;
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
/// P[X = x] = \frac{1 - e^{-1/scale}}{1 + e^{-1/scale}} e^{-|x|/scale}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{Z}(0, scale)
/// ```
///
/// # Citation
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/abs/2004.00010)
pub fn sample_discrete_laplace(scale: RBig) -> Fallible<IBig> {
    sample_discrete_laplace_with(scale, &mut SamplerEntropy::new())
}

fn sample_discrete_laplace_with(scale: RBig, entropy: &mut impl EntropySource) -> Fallible<IBig> {
    if scale.is_zero() {
        return Ok(0.into());
    }
    if scale < RBig::ZERO {
        return fallible!(FailedFunction, "scale must be nonnegative");
    }
    let (numer, denom) = scale.into_parts();
    let inv_scale = RBig::from_parts(denom.as_ibig().clone(), numer.into_parts().1);

    sample_discrete_laplace_from_inverse(&inv_scale, entropy)
}

fn sample_discrete_laplace_from_inverse(
    inv_scale: &RBig,
    entropy: &mut impl EntropySource,
) -> Fallible<IBig> {
    loop {
        let positive = entropy.coin()?;
        let magnitude = sample_geometric_exp_fast_with(inv_scale.clone(), entropy)?
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
    sample_discrete_gaussian_with(scale, &mut SamplerEntropy::new())
}

fn sample_discrete_gaussian_with(scale: RBig, entropy: &mut impl EntropySource) -> Fallible<IBig> {
    if scale.is_zero() {
        return Ok(IBig::ZERO);
    }
    let envelope = GaussianEnvelope::new(scale)?;
    loop {
        let candidate = sample_discrete_laplace_from_inverse(&envelope.inv_t, entropy)?;
        if sample_bernoulli_exp_ratio(
            envelope.bias_numerator(&candidate),
            &envelope.denominator,
            entropy,
        )? {
            return Ok(candidate);
        }
    }
}

/// Integer coefficients for the unchanged CKS Gaussian rejection exponent.
struct GaussianEnvelope {
    inv_t: RBig,
    a_squared: UBig,
    coefficient: UBig,
    denominator: UBig,
}

impl GaussianEnvelope {
    fn new(scale: RBig) -> Fallible<Self> {
        if scale <= RBig::ZERO {
            return fallible!(FailedFunction, "Gaussian envelope needs positive scale");
        }
        let t = (scale.clone().floor() + 1i8).into_parts().1;
        let (a, b) = scale.into_parts();
        let a_squared = a.into_parts().1.pow(2);
        let coefficient = b.pow(2) * &t;
        let denominator = (&a_squared * &coefficient * &t) << 1usize;
        let inv_t = RBig::from_parts(1.into(), t);
        Ok(Self {
            inv_t,
            a_squared,
            coefficient,
            denominator,
        })
    }

    fn bias_numerator(&self, candidate: &IBig) -> UBig {
        // For sigma=a/b and t=floor(sigma)+1:
        // (|c|-sigma^2/t)^2/(2sigma^2)
        //   = (|c| b^2 t-a^2)^2 / (2 a^2 b^2 t^2).
        // Keep this ratio unreduced: constructing RBig would perform a wide gcd.
        let delta = candidate.abs() * &self.coefficient - self.a_squared.as_ibig();
        delta.into_parts().1.pow(2)
    }
}
