//! Convert between noise scales and accuracies.

#[cfg(feature="ffi")]
mod ffi;

use std::f64::consts::SQRT_2;

use num::{Float, One, Zero};
use opendp_derive::bootstrap;
use statrs::function::erf::erf_inv;

use crate::error::Fallible;
use crate::traits::InfCast;
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
pub fn laplacian_scale_to_accuracy<T: Float + Zero + One + Debug>(scale: T, alpha: T) -> Fallible<T> {
    if scale.is_sign_negative() {
        return fallible!(InvalidDistance, "scale may not be negative")
    }
    if alpha <= T::zero() || T::one() < alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1]")
    }
    Ok(scale * alpha.recip().ln())
}

#[bootstrap(arguments(scale(c_type = "void *"), alpha(c_type = "void *")))]
/// Convert a discrete Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
/// 
/// $\alpha = P[Y \ge accuracy]$, where $Y = |X - z|$, and $X \sim \mathcal{L}_{Z}(0, scale)$.
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
pub fn discrete_laplacian_scale_to_accuracy<T: Float + Zero + One + Debug>(scale: T, alpha: T) -> Fallible<T> {
    if scale.is_sign_negative() {
        return fallible!(InvalidDistance, "scale may not be negative")
    }
    if alpha <= T::zero() || T::one() < alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1]")
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
pub fn accuracy_to_laplacian_scale<T: Float + Zero + One + Debug>(accuracy: T, alpha: T) -> Fallible<T> {
    if accuracy.is_sign_negative() {
        return fallible!(InvalidDistance, "accuracy may not be negative")
    }
    if alpha <= T::zero() || T::one() <= alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1)")
    }
    Ok(accuracy / alpha.recip().ln())
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
pub fn accuracy_to_discrete_laplacian_scale<T: Float + Zero + One + Debug>(accuracy: T, alpha: T) -> Fallible<T> {
    // the continuous laplacian scale is an upper bound
    let mut s_max = accuracy_to_laplacian_scale(accuracy, alpha)?;
    let mut s_min = T::zero();

    let _2 = T::one() + T::one();

    // run binary search to find ideal scale
    loop {
        let diff = s_max - s_min;
        let s_mid = s_min + diff / _2;

        if s_mid == s_max || s_mid == s_min {
            return Ok(s_max)
        }

        if discrete_laplacian_scale_to_accuracy(s_mid, alpha)? >= accuracy {
            s_max = s_mid;
        } else {
            s_min = s_mid;
        }
    }
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
    where f64: InfCast<T>, T: InfCast<f64> {
    let scale = f64::inf_cast(scale)?;
    let alpha = f64::inf_cast(alpha)?;
    if scale.is_sign_negative() {
        return fallible!(InvalidDistance, "scale may not be negative")
    }
    if alpha <= 0. || 1. < alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1]")
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
    where f64: InfCast<T>, T: InfCast<f64> {
    let scale = f64::inf_cast(scale)?;
    let alpha = f64::inf_cast(alpha)?;
    
    let mut total = (1. - alpha) * dg_normalization_term(scale);
    let mut i = 0;
    total -= dg_pdf(i, scale);
    while total.is_sign_positive() {
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
    where f64: InfCast<T>, T: InfCast<f64> {
    let accuracy = f64::inf_cast(accuracy)?;
    let alpha = f64::inf_cast(alpha)?;
    if accuracy.is_sign_negative() {
        return fallible!(InvalidDistance, "accuracy may not be negative")
    }
    if alpha <= 0. || 1. <= alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1)")
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
    where f64: InfCast<T>, T: InfCast<f64> {
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
            return T::inf_cast(s_max)
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
            return total
        }
        total += density_i;
    }
}


#[cfg(all(test, feature="untrusted", feature="use-mpfr"))]
pub mod test {
    use std::fmt::Debug;
    use std::ops::{Mul, Sub};

    use super::*;
    use crate::measurements::{make_base_laplace, make_base_gaussian, make_base_discrete_laplace, make_base_discrete_gaussian};
    use crate::error::ExplainUnwrap;
    use crate::domains::AllDomain;
    use crate::measures::ZeroConcentratedDivergence;

    #[test]
    fn test_comparison() -> Fallible<()> {
        let alpha = 0.05;
        let scale = 20.;
        let accuracy = 20.;

        let c_acc = laplacian_scale_to_accuracy(scale, alpha)?;
        let d_acc = discrete_laplacian_scale_to_accuracy(scale, alpha)?;
        assert!(c_acc < d_acc);
        println!("lap cont accuracy: {}", c_acc);
        println!("lap disc accuracy: {}", d_acc);

        let c_scale = accuracy_to_laplacian_scale(accuracy, alpha)?;
        let d_scale = accuracy_to_discrete_laplacian_scale(accuracy, alpha)?;
        assert!(c_scale > d_scale);
        println!("lap cont scale: {}", c_scale);
        println!("lap disc scale: {}", d_scale);

        let c_acc = gaussian_scale_to_accuracy(scale, alpha)?;
        let d_acc = discrete_gaussian_scale_to_accuracy(scale, alpha)?;
        assert!(c_acc < d_acc);
        println!("gauss cont accuracy: {}", c_acc);
        println!("gauss disc accuracy: {}", d_acc);

        let c_scale = accuracy_to_gaussian_scale(accuracy, alpha)?;
        let d_scale = accuracy_to_discrete_gaussian_scale(accuracy, alpha)?;
        assert!(c_scale > d_scale);
        println!("gauss cont scale: {}", c_scale);
        println!("gauss disc scale: {}", d_scale);
        Ok(())
    }

    fn print_statement<T: Copy + Debug + One + From<i8> + Sub<Output=T> + Mul<Output=T>>(dist: &str, scale: T, accuracy: T, alpha: T) {
        let _100 = T::from(100);
        println!("When the {dist} scale is {scale:?}, the DP estimate differs from the true value \
                    by no more than {accuracy:?} at a level-alpha of {alpha:?}, \
                    or with (1 - {alpha:?})100% = {perc:.2?}% confidence.",
                 dist = dist, scale = scale,
                 accuracy = accuracy, alpha = alpha, perc = (T::one() - alpha) * _100);
    }

    #[test]
    fn test_laplacian_scale_to_accuracy() -> Fallible<()> {
        macro_rules! check_laplacian_scale_to_accuracy {(scale=$scale:literal, alpha=$alpha:literal) =>
            (print_statement("laplacian", $scale, laplacian_scale_to_accuracy($scale, $alpha)?, $alpha))
        }
        check_laplacian_scale_to_accuracy!(scale=1., alpha=0.05);
        check_laplacian_scale_to_accuracy!(scale=2., alpha=0.05);
        check_laplacian_scale_to_accuracy!(scale=0., alpha=0.55);
        Ok(())
    }

    #[test]
    pub fn test_accuracy_to_laplacian_scale() -> Fallible<()> {
        macro_rules! check_accuracy_to_laplacian_scale {(accuracy=$accuracy:literal, alpha=$alpha:literal) =>
            (print_statement("laplacian", accuracy_to_laplacian_scale($accuracy, $alpha)?, $accuracy, $alpha))
        }
        check_accuracy_to_laplacian_scale!(accuracy=1., alpha=0.05);
        check_accuracy_to_laplacian_scale!(accuracy=2., alpha=0.05);
        check_accuracy_to_laplacian_scale!(accuracy=0.01, alpha=0.1);
        Ok(())
    }

    #[test]
    pub fn test_gaussian_scale_to_accuracy() -> Fallible<()> {
        macro_rules! check_gaussian_scale_to_accuracy {(scale=$scale:literal, alpha=$alpha:literal) =>
            (print_statement("gaussian", $scale, gaussian_scale_to_accuracy($scale, $alpha)?, $alpha))
        }

        check_gaussian_scale_to_accuracy!(scale=1., alpha=0.05);
        check_gaussian_scale_to_accuracy!(scale=2., alpha=0.10);
        check_gaussian_scale_to_accuracy!(scale=3., alpha=0.55);
        Ok(())
    }

    #[test]
    pub fn test_accuracy_to_gaussian_scale() -> Fallible<()> {
        macro_rules! check_accuracy_to_gaussian_scale {(accuracy=$accuracy:literal, alpha=$alpha:literal) =>
            (print_statement("gaussian", accuracy_to_gaussian_scale($accuracy, $alpha)?, $accuracy, $alpha))
        }
        check_accuracy_to_gaussian_scale!(accuracy=1., alpha=0.05);
        check_accuracy_to_gaussian_scale!(accuracy=2., alpha=0.05);
        check_accuracy_to_gaussian_scale!(accuracy=1.2, alpha=0.1);
        Ok(())
    }


    #[test]
    fn test_relative_laplacian_scale_to_accuracy() -> Fallible<()> {
        // fix the scale.
        // you get a tighter accuracy interval when you require greater statistical significance
        // a higher confidence accuracy interval is wider than a lower confidence accuracy interval
        assert!(laplacian_scale_to_accuracy(1., 0.05)? // 95% confidence
            > laplacian_scale_to_accuracy(1., 0.06)?); // 94% confidence

        // fix the alpha/statistical significance.
        // you get a tighter accuracy interval when there is less noise
        // a less noisy sample produces a tighter/smaller accuracy interval
        assert!(laplacian_scale_to_accuracy(2., 0.05)? // 95% confidence
            > laplacian_scale_to_accuracy(1., 0.05)?); // 95% confidence

        Ok(())
    }

    #[test]
    pub fn test_relative_accuracy_to_laplacian_scale() -> Fallible<()> {
        // fix the size of the accuracy interval.
        // if I want more confidence in the result, then I should have less noise
        // you get a larger noise scale when you require greater statistical significance
        // a higher confidence laplace scale is smaller than a lower confidence laplace scale
        assert!(accuracy_to_laplacian_scale(1., 0.05)? // 95% confidence
            < accuracy_to_laplacian_scale(1., 0.06)?); // 94% confidence

        // fix alpha/statistical significance.
        // you get a larger noise scale when there is a wider accuracy interval
        assert!(accuracy_to_laplacian_scale(2., 0.05)? // 95% confidence
            > accuracy_to_laplacian_scale(1., 0.05)?); // 95% confidence

        Ok(())
    }

    #[test]
    pub fn test_relative_gaussian_scale_to_accuracy() -> Fallible<()> {
        // fix the scale.
        // you get a tighter accuracy interval when you require greater statistical significance
        // a higher confidence accuracy interval is wider than a lower confidence accuracy interval
        assert!(gaussian_scale_to_accuracy(1., 0.05)? // 95% confidence
            > gaussian_scale_to_accuracy(1., 0.06)?); // 94% confidence

        // fix the alpha/statistical significance.
        // you get a tighter accuracy interval when there is less noise
        // a less noisy sample produces a tighter/smaller accuracy interval
        assert!(gaussian_scale_to_accuracy(2., 0.05)? // 95% confidence
            > gaussian_scale_to_accuracy(1., 0.05)?); // 95% confidence

        Ok(())
    }

    #[test]
    pub fn test_relative_accuracy_to_gaussian_scale() -> Fallible<()> {
        // fix the size of the accuracy interval.
        // if I want more confidence in the result, then I should have less noise
        // you get a larger noise scale when you require greater statistical significance
        // a higher confidence noise scale is smaller than a lower confidence noise scale
        assert!(accuracy_to_gaussian_scale(1., 0.05)? // 95% confidence
            < accuracy_to_gaussian_scale(1., 0.06)?); // 94% confidence

        // fix alpha/statistical significance.
        // you get a larger noise scale when there is a wider accuracy interval
        assert!(accuracy_to_gaussian_scale(2., 0.05)? // 95% confidence
            > accuracy_to_gaussian_scale(1., 0.05)?); // 95% confidence
        Ok(())
    }

    #[test]
    pub fn test_empirical_laplace_accuracy() -> Fallible<()> {
        let accuracy = 1.0;
        let theoretical_alpha = 0.05;
        let scale = accuracy_to_laplacian_scale(accuracy, theoretical_alpha)?;
        let base_laplace = make_base_laplace::<AllDomain<f64>>(scale, Some(-100))?;
        let n = 50_000;
        let empirical_alpha = (0..n)
            .filter(|_| base_laplace.invoke(&0.0).unwrap_test().abs() > accuracy)
            .count() as f64 / n as f64;

        println!("Laplacian significance levels/alpha");
        println!("Theoretical: {:?}", theoretical_alpha);
        println!("Empirical:   {:?}", empirical_alpha);
        assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
        Ok(())
    }

    #[test]
    pub fn test_empirical_gaussian_accuracy() -> Fallible<()> {
        let accuracy = 1.0;
        let theoretical_alpha = 0.05;
        let scale = accuracy_to_gaussian_scale(accuracy, theoretical_alpha)?;
        let base_gaussian = make_base_gaussian::<AllDomain<f64>, ZeroConcentratedDivergence<_>>(scale, Some(-100))?;
        let n = 50_000;
        let empirical_alpha = (0..n)
            .filter(|_| base_gaussian.invoke(&0.0).unwrap_test().abs() > accuracy)
            .count() as f64 / n as f64;

        println!("Gaussian significance levels/alpha");
        println!("Theoretical: {:?}", theoretical_alpha);
        println!("Empirical:   {:?}", empirical_alpha);
        assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
        Ok(())
    }

    #[test]
    pub fn test_empirical_discrete_laplace_accuracy() -> Fallible<()> {
        let accuracy = 25;
        let theoretical_alpha = 0.05;
        let scale = accuracy_to_discrete_laplacian_scale(accuracy as f64, theoretical_alpha)?;
        println!("scale: {scale}");
        let base_dl = make_base_discrete_laplace::<AllDomain<i8>, f64>(scale)?;
        let n = 50_000;
        let empirical_alpha = (0..n)
            .filter(|_| base_dl.invoke(&0).unwrap_test().clamp(-127, 127).abs() >= accuracy)
            .count() as f64 / n as f64;

        println!("Discrete laplace significance levels/alpha");
        println!("Theoretical: {:?}", theoretical_alpha);
        println!("Empirical:   {:?}", empirical_alpha);
        assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
        Ok(())
    }

    #[test]
    pub fn test_empirical_discrete_gaussian_accuracy() -> Fallible<()> {
        let accuracy = 25;
        let theoretical_alpha = 0.05;
        let scale = accuracy_to_discrete_gaussian_scale(accuracy as f64, theoretical_alpha)?;
        // let scale = 12.503562372734077;

        println!("scale: {}", scale);
        let base_dg = make_base_discrete_gaussian::<AllDomain<i8>, ZeroConcentratedDivergence<f64>, i32>(scale)?;
        let n = 50_000;
        let empirical_alpha = (0..n)
            .filter(|_| base_dg.invoke(&0).unwrap_test().clamp(-127, 127).abs() >= accuracy)
            .count() as f64 / n as f64;

        println!("Discrete gaussian significance levels/alpha");
        println!("Theoretical: {:?}", theoretical_alpha);
        println!("Empirical:   {:?}", empirical_alpha);
        assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
        Ok(())
    }

    #[test]
    pub fn test_roundtrip() -> Fallible<()> {
        let accuracy = 1.;
        let alpha = 0.05;
        let accuracy_2 = gaussian_scale_to_accuracy(accuracy_to_gaussian_scale(accuracy, alpha)?, alpha)?;
        assert!((accuracy - accuracy_2).abs() < 1e-8);

        let accuracy_2 = laplacian_scale_to_accuracy(accuracy_to_laplacian_scale(accuracy, alpha)?, alpha)?;
        assert!((accuracy - accuracy_2).abs() < 1e-8);
        Ok(())
    }
}