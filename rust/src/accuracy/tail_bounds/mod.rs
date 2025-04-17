use dashu::{base::Inverse, integer::UBig, rational::RBig};
use num::Zero;
use statrs::function::erf::erfc;

use dashu::integer::IBig;

use crate::{
    error::Fallible,
    traits::{InfAdd, InfCast, InfDiv, InfExp, InfMul, InfPowI, InfSqrt, NextFloat},
};

#[cfg(all(test, feature = "contrib"))]
mod test;

/// Computes the probability of sampling a value greater than `t` from the continuous laplace distribution.
///
/// Arithmetic is controlled such that the resulting probability can only ever be slightly over-estimated due to numerical inaccuracy.
///
/// # Proof definition
/// Returns `Ok(out)`, where `out` does not underestimate $\Pr[X > t]$
/// for $X \sim \mathcal{L}_\mathbb{R}(0, s)$, assuming $t > 0$,
/// or `Err(e)` if any numerical computation overflows.
///
/// $\mathcal{L}_\mathbb{R}(0, s)$ is distributed as follows:
/// ```math
/// \forall x \in \mathbb{R}, \quad  
/// P[X = x] = \frac{1}{2 s}e^{-|x|/s}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{R}(0, s)
/// ```
pub fn conservative_continuous_laplacian_tail_to_alpha(scale: RBig, tail: RBig) -> Fallible<f64> {
    // tail and scale division should be big rationals for precision and to avoid overflow
    f64::neg_inf_cast(-tail / scale)?.inf_exp()?.inf_div(&2.0)
}

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
/// P[X = x] = \frac{1 - e^{-1/scale}}{1 + e^{-1/scale}} e^{-|x|/scale}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{Z}(0, scale)
/// ```
pub fn conservative_discrete_laplacian_tail_to_alpha(scale: RBig, tail: UBig) -> Fallible<f64> {
    let numer = f64::inf_cast(-RBig::from(tail) / scale.clone())?.inf_exp()?;
    let denom = f64::neg_inf_cast(RBig::ONE / scale)?
        .neg_inf_exp()?
        .neg_inf_add(&1.)?;
    numer.inf_div(&denom)
}

/// Computes the probability of sampling a value greater than `t` from the discrete laplace distribution.
///
/// Arithmetic is controlled such that the resulting probability can only ever be slightly under-estimated due to numerical inaccuracy.
///
/// # Proof definition
/// Returns `Ok(out)`, where `out` does not overestimate $\Pr[X > t]$
/// for $X \sim \mathcal{L}_\mathbb{Z}(0, scale)$, assuming $t > 0$,
/// or `Err(e)` if any numerical computation overflows.
///
/// $\mathcal{L}_\mathbb{Z}(0, scale)$ is distributed as follows:
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{1 - e^{-1/scale}}{1 + e^{-1/scale}} e^{-|x|/scale}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{Z}(0, scale)
/// ```
pub fn conservative_discrete_laplacian_tail_to_alpha_lower(
    scale: RBig,
    tail: UBig,
) -> Fallible<f64> {
    let numer = f64::neg_inf_cast(-RBig::from(tail) / scale.clone())?.neg_inf_exp()?;
    let denom = f64::inf_cast(RBig::ONE / scale)?.inf_exp()?.inf_add(&1.)?;
    numer.neg_inf_div(&denom)
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
pub fn conservative_discrete_gaussian_tail_to_alpha(scale: RBig, tail: UBig) -> Fallible<f64> {
    // where tail = m - 1
    conservative_continuous_gaussian_tail_to_alpha(scale, RBig::from(tail))
}

/// Computes the probability of sampling a value greater than `t` from the discrete gaussian distribution.
///
/// Arithmetic is controlled such that the resulting probability can only ever be under-estimated due to numerical inaccuracy.
///
/// # Citations
/// * Fact 20, Proposition 25: [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf)
///
/// # Proof definition
/// Returns `Ok(out)`, where `out` does not overestimate $\Pr[X > t]$
/// for $X \sim \mathcal{N}_\mathbb{Z}(0, scale)$, assuming $t > 0$,
/// or `Err(e)` if any numerical computation overflows.
///
/// $\mathcal{N}_\mathbb{Z}(0, scale)$ is distributed as follows:
/// ```math
/// \forall x \in \mathbb{Z}, \quad  
/// P[X = x] = \frac{e^{-\frac{x^2}{2\sigma^2}}}{\sum_{y\in\mathbb{Z}}e^{-\frac{y^2}{2\sigma^2}}}, \quad
/// \text{where } X \sim \mathcal{N}_\mathbb{Z}(0, \sigma^2)
/// ```
pub fn conservative_discrete_gaussian_tail_to_alpha_lower(
    scale: RBig,
    tail: UBig,
) -> Fallible<f64> {
    // where tail = m - 1

    let pi = std::f64::consts::PI;
    let frac_1_pi = std::f64::consts::FRAC_1_PI;

    let twice_scale2 = RBig::from(2) * scale.clone().pow(2);

    let exp_minus_2pi2_scale2_upper = pi
        .neg_inf_powi(IBig::from(2))?
        .inf_mul(&f64::inf_cast(-twice_scale2.clone())?)?
        .inf_exp()?;

    let exp_minus_inverse_twice_scale2_upper = f64::inf_cast(-twice_scale2.inv())?.inf_exp()?;

    let inv_scale_sqrt_tau = frac_1_pi
        .inf_div(&f64::inf_cast(2)?)?
        .inf_sqrt()?
        .inf_mul(&f64::inf_cast(scale.clone().inv())?)?;

    // Compute expressions of which the minimum will be the denominator
    let denominator_a = exp_minus_2pi2_scale2_upper
        .inf_mul(&2.0)?
        .inf_add(&1.0)?
        .inf_add(&exp_minus_2pi2_scale2_upper.inf_mul(&inv_scale_sqrt_tau)?)?;

    let denominator_b = exp_minus_inverse_twice_scale2_upper
        .inf_mul(&2.0)?
        .inf_add(&1.0)?
        .inf_mul(&inv_scale_sqrt_tau)?
        .inf_add(&exp_minus_inverse_twice_scale2_upper)?;

    let denominator_min = denominator_a.min(denominator_b);

    // compute tail bound
    let continuous_tail_bound = conservative_continuous_gaussian_tail_to_alpha_lower(
        scale,
        RBig::from(tail + UBig::from(1u8)),
    )?;
    continuous_tail_bound.neg_inf_div(&denominator_min)
}

// pub fn conservative_discrete_gaussian_tail_to_alpha_lower(scale: f64, tail: u32) -> Fallible<f64> {
//     // where tail = m - 1
//
//     let pi = std::f64::consts::PI;
//     let tau = std::f64::consts::TAU;
//
//     // require scale >= 1/sqrt(2*pi), since this is a requirement of Proposition 25 cited above
//     let scale_squared = scale.neg_inf_powi(IBig::from(2))?;
//     let tau_inverse = 1.0.neg_inf_div(&tau)?;
//     let difference = scale_squared.neg_inf_sub(&tau_inverse)?;
//
//     if difference < 0.0 {
//         return Err(err!(
//             FailedFunction,
//             format!("scale must be at least 1/sqrt(2*pi), got {}", scale)
//         ));
//     }
//
//     // compute tail bound
//     let t = pi.neg_inf_mul(&scale)?;
//     let t = t.neg_inf_powi(IBig::from(2))?;
//     let t = t.inf_mul(&-2.0)?;
//     let t = t.inf_exp()?;
//     let t = t.inf_mul(&3.0)?;
//     let denominator = t.inf_add(&1.0)?;
//     let tail_plus_one = tail.inf_add(&1)?;
//     let tail_bound = conservative_continuous_gaussian_tail_to_alpha_lower(
//         scale,
//         f64::neg_inf_cast(tail_plus_one)?,
//     )?;
//     tail_bound.neg_inf_div(&denominator)
// }

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
pub fn conservative_continuous_gaussian_tail_to_alpha(scale: RBig, tail: RBig) -> Fallible<f64> {
    // the SQRT_2 constant is already rounded down
    let sqrt_2_ceil: f64 = std::f64::consts::SQRT_2.next_up_();

    // tail and scale division should be big rationals for precision and to avoid overflow
    let t = f64::neg_inf_cast(tail / scale)?.neg_inf_div(&sqrt_2_ceil)?;
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

/// Computes the probability of sampling a value greater than or equal to `t` from the continuous gaussian distribution.
///
/// Arithmetic is controlled such that the resulting probability can only ever be slightly under-estimated due to numerical inaccuracy.
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
pub fn conservative_continuous_gaussian_tail_to_alpha_lower(
    scale: RBig,
    tail: RBig,
) -> Fallible<f64> {
    // the SQRT_2 constant is already rounded down
    let sqrt_2_ceil: f64 = std::f64::consts::SQRT_2.next_down_();

    // tail and scale division should be big rationals for precision and to avoid overflow
    let t = f64::inf_cast(tail / scale)?.inf_div(&sqrt_2_ceil)?;
    // round down to nearest smaller f32
    let t = f32::inf_cast(t)? as f64;
    // erfc error is at most 1 f32 ulp (see erfc_err_analysis.py)
    let t = f32::neg_inf_cast(erfc(t))?.next_down_();

    (t as f64).neg_inf_div(&2.0)
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
