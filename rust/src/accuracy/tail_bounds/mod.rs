use num::Zero;
use statrs::function::erf::erfc;

use crate::{
    error::Fallible,
    traits::{InfAdd, InfCast, InfDiv, InfExp, NextFloat},
};

#[cfg(test)]
mod test;

/// Computes the probability of sampling a value greater than `t` from the discrete laplace distribution.
///
/// Arithmetic is controlled such that the resulting probability can only ever be slightly over-estimated due to numerical inaccuracy.
///
/// # Proof definition
/// Returns `Ok(out)`, where `out` does not underestimate $\Pr[X > t]$
/// for $X \sim \mathcal{L}_\mathbb{Z}(0, scale)$, assuming $t > 0$,
/// or `Err(e)` if any numerical computation overflows.
///
/// $\mathcal{L}_\mathbb{Z}(0, scale)$ is distributed as follows:
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{e^{-1/scale} - 1}{e^{-1/scale} + 1} e^{-|x|/scale}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{Z}(0, scale)
/// ```
pub fn conservative_discrete_laplacian_tail_to_alpha(scale: f64, tail: u32) -> Fallible<f64> {
    let t = f64::neg_inf_cast(tail)?;
    let numer = t.neg_inf_div(&-scale)?.inf_exp()?;
    let denom = scale.recip().neg_inf_exp()?.neg_inf_add(&1.)?;
    numer.inf_div(&denom)
}

/// Computes the probability of sampling a value greater than `t` from the discrete gaussian distribution.
///
/// Arithmetic is controlled such that the resulting probability can only ever be slightly over-estimated due to numerical inaccuracy.
///
/// # Citations
/// * Proposition 25: [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf)
///
/// # Proof definition
/// Returns `Ok(out)`, where `out` does not underestimate $\Pr[X > t]$
/// for $X \sim \mathcal{N}_\mathbb{Z}(0, scale)$, assuming $t > 0$,
/// or `Err(e)` if any numerical computation overflows.
///
/// $\mathcal{N}_\mathbb{Z}(0, scale)$ is distributed as follows:
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{e^{-\frac{x^2}{2\sigma^2}}}{\sum_{y\in\mathbb{Z}}e^{-\frac{y^2}{2\sigma^2}}}, \quad
/// \text{where } X \sim \mathcal{N}_\mathbb{Z}(0, \sigma^2)
/// ```
pub fn conservative_discrete_gaussian_tail_to_alpha(scale: f64, tail: u32) -> Fallible<f64> {
    // where tail = m - 1
    conservative_continuous_gaussian_tail_to_alpha(scale, f64::neg_inf_cast(tail)?)
}

/// Computes the probability of sampling a value greater than or equal to `t` from the continuous gaussian distribution.
///
/// Arithmetic is controlled such that the resulting probability can only ever be slightly over-estimated due to numerical inaccuracy.
///
/// # Proof definition
/// Returns `Ok(out)`, where `out` does not underestimate $\Pr[X > t]$
/// for $X \sim \mathcal{N}(0, scale)$, assuming $t > 0$,
/// or `Err(e)` if any numerical computation overflows.
///
/// X is distributed $\mathcal{N}(0, scale)$ with probability density:
/// ```math
/// f(x) = \frac{1}{\sigma \sqrt{2 \pi}} e^{-\frac{1}{2}\left( \frac{x - \mu}{\sigma}\right)^2}
/// ```
pub fn conservative_continuous_gaussian_tail_to_alpha(scale: f64, tail: f64) -> Fallible<f64> {
    // the SQRT_2 constant is already rounded down
    let SQRT_2_CEIL: f64 = std::f64::consts::SQRT_2.next_up_();

    let t = tail.neg_inf_div(&scale)?.neg_inf_div(&SQRT_2_CEIL)?;
    // round down to nearest smaller f32
    let t = f32::neg_inf_cast(t)? as f64;
    // erfc error is at most 1 f32 ulp (see erfc_err_analysis.py)
    let t = f32::inf_cast(erfc(t))?.next_up_();

    (t as f64).inf_div(&2.0)

    // this bound does the same thing,
    // but is loose by a factor of 10 on common workloads
    // // e^{-(t / scale)^2 / 2}
    // t.neg_inf_div(&scale)?
    //     .neg_inf_powi(ibig!(2))?
    //     .neg_inf_div(&2.0)?
    //     .neg()
    //     .inf_exp()
}

pub(super) fn dg_pdf(x: i32, scale: f64) -> f64 {
    (-(x as f64 / scale).powi(2) / 2.).exp()
}

pub(super) fn dg_normalization_term(scale: f64) -> f64 {
    let mut i = 0;
    let mut total = dg_pdf(i, scale);
    loop {
        i += 1;
        let density_i = 2. * dg_pdf(i, scale);
        if density_i.is_zero() {
            return total;
        }
        total += density_i;
    }
}
