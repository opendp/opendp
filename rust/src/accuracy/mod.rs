//! Convert between noise scales and accuracies.

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "polars")]
mod polars;

use std::f64::consts::SQRT_2;

use num::{Float, One, Zero};
use opendp_derive::bootstrap;
use statrs::function::erf::erf_inv;

use crate::error::Fallible;
use crate::traits::{InfAdd, InfCast, InfDiv, InfExp};
use std::fmt::Debug;

#[bootstrap(arguments(scale(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
///
/// # Arguments
/// * `scale` - Laplacian noise scale.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `scale` and `alpha`
pub fn laplacian_scale_to_accuracy<T: Float + Zero + One + Debug>(
    scale: T,
    alpha: T,
) -> Fallible<T> {
    if scale.is_sign_negative() {
        return fallible!(InvalidDistance, "scale ({:?}) may not be negative", scale);
    }
    if alpha <= T::zero() || T::one() < alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1]", alpha);
    }
    Ok(-scale * alpha.ln())
}

#[bootstrap(arguments(scale(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a discrete Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
///
/// $\alpha = P[Y \ge accuracy]$, where $Y = | X - z |$, and $X \sim \mathcal{L}_{Z}(0, scale)$.
/// That is, $X$ is a discrete Laplace random variable and $Y$ is the distribution of the errors.
///
/// This function returns a float accuracy.
/// You can take the floor without affecting the coverage probability.
///
/// # Arguments
/// * `scale` - Discrete Laplacian noise scale.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `scale` and `alpha`
pub fn discrete_laplacian_scale_to_accuracy<T: Float + Zero + One + Debug>(
    scale: T,
    alpha: T,
) -> Fallible<T> {
    if scale.is_sign_negative() {
        return fallible!(InvalidDistance, "scale ({:?}) may not be negative", scale);
    }
    if alpha <= T::zero() || T::one() < alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1]", alpha);
    }

    let _1 = T::one();
    let _2 = _1 + _1;

    // somewhere between 1 and 2
    // term = 2 / (exp(1/scale) + 1)
    let term = _2 / (scale.recip().exp() + _1);

    // scale * ln(1/α * term) + 1
    Ok(scale * (alpha.recip() * term).ln() + _1)
}

#[bootstrap(arguments(accuracy(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a desired `accuracy` (tolerance) into a Laplacian noise scale at a statistical significance level `alpha`.
///
/// # Arguments
/// * `accuracy` - Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `accuracy` and `alpha`
///
/// # Returns
/// Laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
pub fn accuracy_to_laplacian_scale<T: Float + Zero + One + Debug>(
    accuracy: T,
    alpha: T,
) -> Fallible<T> {
    if accuracy.is_sign_negative() {
        return fallible!(
            InvalidDistance,
            "accuracy ({:?}) may not be negative",
            accuracy
        );
    }
    if alpha <= T::zero() || T::one() <= alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1)", alpha);
    }
    Ok(-accuracy / alpha.ln())
}

#[bootstrap(arguments(accuracy(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a desired `accuracy` (tolerance) into a discrete Laplacian noise scale at a statistical significance level `alpha`.
///
/// # Arguments
/// * `accuracy` - Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `accuracy` and `alpha`
///
/// # Returns
/// Discrete laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
pub fn accuracy_to_discrete_laplacian_scale<T: Float + Zero + One + Debug>(
    accuracy: T,
    alpha: T,
) -> Fallible<T> {
    // the continuous laplacian scale is an upper bound
    let mut s_max = accuracy_to_laplacian_scale(accuracy, alpha)?;
    let mut s_min = T::zero();

    let _2 = T::one() + T::one();

    // run binary search to find ideal scale
    loop {
        let diff = s_max - s_min;
        let s_mid = s_min + diff / _2;

        if s_mid == s_max || s_mid == s_min {
            return Ok(s_max);
        }

        if discrete_laplacian_scale_to_accuracy(s_mid, alpha)? >= accuracy {
            s_max = s_mid;
        } else {
            s_min = s_mid;
        }
    }
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
/// P[X = x] = \frac{e^{-1/scale} - 1}{e^{-1/scale} + 1} e^{-|x|/scale}, \quad
/// \text{where } X \sim \mathcal{L}_\mathbb{Z}(0, scale)
/// ```
pub fn integrate_discrete_laplacian_tail(scale: f64, t: u32) -> Fallible<f64> {
    let t = f64::neg_inf_cast(t)?;
    let numer = t.neg_inf_div(&-scale)?.inf_exp()?;
    let denom = scale.recip().neg_inf_exp()?.neg_inf_add(&1.)?;

    numer.inf_div(&denom)
}

#[bootstrap(arguments(scale(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
///
/// # Arguments
/// * `scale` - Gaussian noise scale.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `scale` and `alpha`
pub fn gaussian_scale_to_accuracy<T>(scale: T, alpha: T) -> Fallible<T>
where
    f64: InfCast<T>,
    T: InfCast<f64>,
{
    let scale = f64::inf_cast(scale)?;
    let alpha = f64::inf_cast(alpha)?;
    if scale.is_sign_negative() {
        return fallible!(InvalidDistance, "scale ({:?}) may not be negative", scale);
    }
    if alpha <= 0. || 1. < alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1]", alpha);
    }
    T::inf_cast(scale * SQRT_2 * erf_inv(1. - alpha))
}

#[bootstrap(arguments(scale(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a discrete gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
///
/// # Arguments
/// * `scale` - Gaussian noise scale.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `scale` and `alpha`
pub fn discrete_gaussian_scale_to_accuracy<T>(scale: T, alpha: T) -> Fallible<T>
where
    f64: InfCast<T>,
    T: InfCast<f64>,
{
    let scale = f64::inf_cast(scale)?;
    let alpha = f64::inf_cast(alpha)?;

    let mut total = (1. - alpha) * dg_normalization_term(scale);
    let mut i = 0;
    total -= dg_pdf(i, scale);
    while total > 0. {
        i += 1;
        let dens = 2. * dg_pdf(i, scale);
        if dens.is_zero() {
            return fallible!(FailedFunction, "could not determine accuracy");
        }
        total -= dens;
    }
    T::inf_cast((i + 1) as f64)
}

#[bootstrap(arguments(accuracy(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a desired `accuracy` (tolerance) into a gaussian noise scale at a statistical significance level `alpha`.
///
/// # Arguments
/// * `accuracy` - Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `accuracy` and `alpha`
pub fn accuracy_to_gaussian_scale<T>(accuracy: T, alpha: T) -> Fallible<T>
where
    f64: InfCast<T>,
    T: InfCast<f64>,
{
    let accuracy = f64::inf_cast(accuracy)?;
    let alpha = f64::inf_cast(alpha)?;
    if accuracy.is_sign_negative() {
        return fallible!(
            InvalidDistance,
            "accuracy ({:?}) may not be negative",
            accuracy
        );
    }
    if alpha <= 0. || 1. <= alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1)", alpha);
    }
    T::inf_cast(accuracy / SQRT_2 / erf_inv(1. - alpha))
}

#[bootstrap(arguments(accuracy(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a desired `accuracy` (tolerance) into a discrete gaussian noise scale at a statistical significance level `alpha`.
///
/// # Arguments
/// * `accuracy` - Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
/// * `alpha` - Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
///
/// # Generics
/// * `T` - Data type of `accuracy` and `alpha`
pub fn accuracy_to_discrete_gaussian_scale<T>(accuracy: T, alpha: T) -> Fallible<T>
where
    f64: InfCast<T>,
    T: InfCast<f64>,
{
    let accuracy = f64::inf_cast(accuracy)?;
    let alpha = f64::inf_cast(alpha)?;

    fn exponential_sum(x: i32, scale: f64) -> f64 {
        (1..x).fold(dg_pdf(0, scale), |sum, i| sum + 2. * dg_pdf(i, scale))
    }

    let mut s_max: f64 = accuracy_to_gaussian_scale::<f64>(accuracy, alpha)?;
    let mut s_min = 0.;

    // run binary search to find ideal scale
    loop {
        let diff = s_max - s_min;
        let s_mid = s_min + diff / 2.;

        if s_mid == s_max || s_mid == s_min {
            return T::inf_cast(s_max);
        }

        let success_prob = exponential_sum(accuracy as i32, s_mid) / dg_normalization_term(s_mid);
        if 1. - alpha > success_prob {
            s_max = s_mid;
        } else {
            s_min = s_mid;
        }
    }
}

fn dg_pdf(x: i32, scale: f64) -> f64 {
    (-(x as f64 / scale).powi(2) / 2.).exp()
}

fn dg_normalization_term(scale: f64) -> f64 {
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

#[cfg(all(test, feature = "untrusted"))]
pub mod test;
