#[cfg(feature="ffi")]
mod ffi;

use std::f64::consts::SQRT_2;

use num::{Float, One, Zero};
use statrs::function::erf::erf_inv;

use crate::error::Fallible;
use crate::traits::InfCast;
use std::fmt::Debug;

pub fn laplacian_scale_to_accuracy<T: Float + Zero + One + Debug>(scale: T, alpha: T) -> Fallible<T> {
    if scale.is_sign_negative() {
        return fallible!(InvalidDistance, "scale may not be negative")
    }
    if alpha <= T::zero() || T::one() < alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1]")
    }
    Ok(scale * alpha.recip().ln())
}

pub fn accuracy_to_laplacian_scale<T: Float + Zero + One + Debug>(accuracy: T, alpha: T) -> Fallible<T> {
    if accuracy.is_sign_negative() {
        return fallible!(InvalidDistance, "accuracy may not be negative")
    }
    if alpha <= T::zero() || T::one() <= alpha {
        return fallible!(InvalidDistance, "alpha ({:?}) must be in (0, 1)")
    }
    Ok(accuracy / alpha.recip().ln())
}

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


#[cfg(test)]
pub mod test {
    use std::fmt::Debug;
    use std::ops::{Mul, Sub};

    use super::*;
    use crate::meas::{make_base_laplace, make_base_gaussian, make_base_discrete_laplace};
    use crate::error::ExplainUnwrap;
    use crate::dom::AllDomain;

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
        let base_laplace = make_base_laplace::<AllDomain<f64>>(scale)?;
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
        let base_gaussian = make_base_gaussian::<AllDomain<f64>>(scale)?;
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
        let scale = accuracy_to_laplacian_scale(accuracy as f64, theoretical_alpha)?;
        println!("scale: {}", scale);
        let base_discrete_laplace = make_base_discrete_laplace::<AllDomain<i8>, f64>(scale, None)?;
        let n = 50_000;
        let empirical_alpha = (0..n)
            .filter(|_| base_discrete_laplace.invoke(&0).unwrap_test().abs() >= accuracy)
            .count() as f64 / n as f64;

        println!("discrete_laplace significance levels/alpha");
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