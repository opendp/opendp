use std::fmt::Debug;
use std::ops::{Mul, Sub};

use super::*;
use crate::domains::AtomDomain;
use crate::error::ExplainUnwrap;
use crate::measurements::{
    make_gaussian, make_scalar_float_gaussian, make_scalar_float_laplace,
    make_scalar_integer_laplace,
};
use crate::measures::ZeroConcentratedDivergence;
use crate::metrics::AbsoluteDistance;

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

fn print_statement<T: Copy + Debug + One + From<i8> + Sub<Output = T> + Mul<Output = T>>(
    dist: &str,
    scale: T,
    accuracy: T,
    alpha: T,
) {
    let _100 = T::from(100);
    println!(
        "When the {dist} scale is {scale:?}, the DP estimate differs from the true value \
                by no more than {accuracy:?} at a level-alpha of {alpha:?}, \
                or with (1 - {alpha:?})100% = {perc:.2?}% confidence.",
        dist = dist,
        scale = scale,
        accuracy = accuracy,
        alpha = alpha,
        perc = (T::one() - alpha) * _100
    );
}

#[test]
fn test_laplacian_scale_to_accuracy() -> Fallible<()> {
    macro_rules! check_laplacian_scale_to_accuracy {
        (scale=$scale:literal, alpha=$alpha:literal) => {
            print_statement(
                "laplacian",
                $scale,
                laplacian_scale_to_accuracy($scale, $alpha)?,
                $alpha,
            )
        };
    }
    check_laplacian_scale_to_accuracy!(scale = 1., alpha = 0.05);
    check_laplacian_scale_to_accuracy!(scale = 2., alpha = 0.05);
    check_laplacian_scale_to_accuracy!(scale = 0., alpha = 0.55);
    Ok(())
}

#[test]
pub fn test_accuracy_to_laplacian_scale() -> Fallible<()> {
    macro_rules! check_accuracy_to_laplacian_scale {
        (accuracy=$accuracy:literal, alpha=$alpha:literal) => {
            print_statement(
                "laplacian",
                accuracy_to_laplacian_scale($accuracy, $alpha)?,
                $accuracy,
                $alpha,
            )
        };
    }
    check_accuracy_to_laplacian_scale!(accuracy = 1., alpha = 0.05);
    check_accuracy_to_laplacian_scale!(accuracy = 2., alpha = 0.05);
    check_accuracy_to_laplacian_scale!(accuracy = 0.01, alpha = 0.1);
    Ok(())
}

#[test]
pub fn test_gaussian_scale_to_accuracy() -> Fallible<()> {
    macro_rules! check_gaussian_scale_to_accuracy {
        (scale=$scale:literal, alpha=$alpha:literal) => {
            print_statement(
                "gaussian",
                $scale,
                gaussian_scale_to_accuracy($scale, $alpha)?,
                $alpha,
            )
        };
    }

    check_gaussian_scale_to_accuracy!(scale = 1., alpha = 0.05);
    check_gaussian_scale_to_accuracy!(scale = 2., alpha = 0.10);
    check_gaussian_scale_to_accuracy!(scale = 3., alpha = 0.55);
    Ok(())
}

#[test]
pub fn test_accuracy_to_gaussian_scale() -> Fallible<()> {
    macro_rules! check_accuracy_to_gaussian_scale {
        (accuracy=$accuracy:literal, alpha=$alpha:literal) => {
            print_statement(
                "gaussian",
                accuracy_to_gaussian_scale($accuracy, $alpha)?,
                $accuracy,
                $alpha,
            )
        };
    }
    check_accuracy_to_gaussian_scale!(accuracy = 1., alpha = 0.05);
    check_accuracy_to_gaussian_scale!(accuracy = 2., alpha = 0.05);
    check_accuracy_to_gaussian_scale!(accuracy = 1.2, alpha = 0.1);
    Ok(())
}

#[test]
fn test_relative_laplacian_scale_to_accuracy() -> Fallible<()> {
    // fix the scale.
    // you get a tighter accuracy interval when you require greater statistical significance
    // a higher confidence accuracy interval is wider than a lower confidence accuracy interval
    assert!(
        laplacian_scale_to_accuracy(1., 0.05)? // 95% confidence
        > laplacian_scale_to_accuracy(1., 0.06)?
    ); // 94% confidence

    // fix the alpha/statistical significance.
    // you get a tighter accuracy interval when there is less noise
    // a less noisy sample produces a tighter/smaller accuracy interval
    assert!(
        laplacian_scale_to_accuracy(2., 0.05)? // 95% confidence
        > laplacian_scale_to_accuracy(1., 0.05)?
    ); // 95% confidence

    Ok(())
}

#[test]
pub fn test_relative_accuracy_to_laplacian_scale() -> Fallible<()> {
    // fix the size of the accuracy interval.
    // if I want more confidence in the result, then I should have less noise
    // you get a larger noise scale when you require greater statistical significance
    // a higher confidence laplace scale is smaller than a lower confidence laplace scale
    assert!(
        accuracy_to_laplacian_scale(1., 0.05)? // 95% confidence
        < accuracy_to_laplacian_scale(1., 0.06)?
    ); // 94% confidence

    // fix alpha/statistical significance.
    // you get a larger noise scale when there is a wider accuracy interval
    assert!(
        accuracy_to_laplacian_scale(2., 0.05)? // 95% confidence
        > accuracy_to_laplacian_scale(1., 0.05)?
    ); // 95% confidence

    Ok(())
}

#[test]
pub fn test_relative_gaussian_scale_to_accuracy() -> Fallible<()> {
    // fix the scale.
    // you get a tighter accuracy interval when you require greater statistical significance
    // a higher confidence accuracy interval is wider than a lower confidence accuracy interval
    assert!(
        gaussian_scale_to_accuracy(1., 0.05)? // 95% confidence
        > gaussian_scale_to_accuracy(1., 0.06)?
    ); // 94% confidence

    // fix the alpha/statistical significance.
    // you get a tighter accuracy interval when there is less noise
    // a less noisy sample produces a tighter/smaller accuracy interval
    assert!(
        gaussian_scale_to_accuracy(2., 0.05)? // 95% confidence
        > gaussian_scale_to_accuracy(1., 0.05)?
    ); // 95% confidence

    Ok(())
}

#[test]
pub fn test_relative_accuracy_to_gaussian_scale() -> Fallible<()> {
    // fix the size of the accuracy interval.
    // if I want more confidence in the result, then I should have less noise
    // you get a larger noise scale when you require greater statistical significance
    // a higher confidence noise scale is smaller than a lower confidence noise scale
    assert!(
        accuracy_to_gaussian_scale(1., 0.05)? // 95% confidence
        < accuracy_to_gaussian_scale(1., 0.06)?
    ); // 94% confidence

    // fix alpha/statistical significance.
    // you get a larger noise scale when there is a wider accuracy interval
    assert!(
        accuracy_to_gaussian_scale(2., 0.05)? // 95% confidence
        > accuracy_to_gaussian_scale(1., 0.05)?
    ); // 95% confidence
    Ok(())
}

#[test]
pub fn test_empirical_laplace_accuracy() -> Fallible<()> {
    let accuracy = 1.0;
    let theoretical_alpha = 0.05;
    let scale = accuracy_to_laplacian_scale(accuracy, theoretical_alpha)?;
    let input_domain = AtomDomain::default();
    let input_metric = AbsoluteDistance::default();
    let laplace = make_scalar_float_laplace(input_domain, input_metric, scale, Some(-100))?;
    let n = 50_000;
    let empirical_alpha = (0..n)
        .filter(|_| laplace.invoke(&0.0).unwrap().abs() > accuracy)
        .count() as f64
        / n as f64;

    println!("Laplacian significance levels/alpha");
    println!("Theoretical: {:?}", theoretical_alpha);
    println!("Empirical:   {:?}", empirical_alpha);
    // this test has a small likelihood of failing
    assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
    Ok(())
}

#[test]
pub fn test_empirical_gaussian_accuracy() -> Fallible<()> {
    let accuracy = 1.0;
    let theoretical_alpha = 0.05;
    let scale = accuracy_to_gaussian_scale(accuracy, theoretical_alpha)?;
    let base_gaussian = make_scalar_float_gaussian::<ZeroConcentratedDivergence, _>(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        scale,
        Some(-100),
    )?;
    let n = 50_000;
    let empirical_alpha = (0..n)
        .filter(|_| base_gaussian.invoke(&0.0).unwrap_test().abs() > accuracy)
        .count() as f64
        / n as f64;

    println!("Gaussian significance levels/alpha");
    println!("Theoretical: {:?}", theoretical_alpha);
    println!("Empirical:   {:?}", empirical_alpha);
    // this test has a small likelihood of failing
    assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
    Ok(())
}

#[test]
pub fn test_empirical_discrete_laplace_accuracy() -> Fallible<()> {
    let accuracy = 25;
    let theoretical_alpha = 0.05;
    let scale = accuracy_to_discrete_laplacian_scale(accuracy as f64, theoretical_alpha)?;
    println!("scale: {scale}");
    let input_domain = AtomDomain::<i32>::default();
    let input_metric = AbsoluteDistance::default();
    let base_dl = make_scalar_integer_laplace(input_domain, input_metric, scale)?;
    let n = 50_000;
    let empirical_alpha = (0..n)
        .filter(|_| base_dl.invoke(&0).unwrap().clamp(-127, 127).abs() >= accuracy)
        .count() as f64
        / n as f64;

    println!("Discrete laplace significance levels/alpha");
    println!("Theoretical: {:?}", theoretical_alpha);
    println!("Empirical:   {:?}", empirical_alpha);
    // this test has a small likelihood of failing
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
    let base_dg = make_gaussian::<_, ZeroConcentratedDivergence, i32>(
        AtomDomain::<i8>::default(),
        AbsoluteDistance::default(),
        scale,
        None,
    )?;
    let n = 50_000;
    let empirical_alpha = (0..n)
        .filter(|_| base_dg.invoke(&0).unwrap().clamp(-127, 127).abs() >= accuracy)
        .count() as f64
        / n as f64;

    println!("Discrete gaussian significance levels/alpha");
    println!("Theoretical: {:?}", theoretical_alpha);
    println!("Empirical:   {:?}", empirical_alpha);
    // this test has a small likelihood of failing
    assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
    Ok(())
}

#[test]
pub fn test_roundtrip() -> Fallible<()> {
    let accuracy = 1.;
    let alpha = 0.05;
    let accuracy_2 =
        gaussian_scale_to_accuracy(accuracy_to_gaussian_scale(accuracy, alpha)?, alpha)?;
    assert!((accuracy - accuracy_2).abs() < 1e-8);

    let accuracy_2 =
        laplacian_scale_to_accuracy(accuracy_to_laplacian_scale(accuracy, alpha)?, alpha)?;
    assert!((accuracy - accuracy_2).abs() < 1e-8);
    Ok(())
}
