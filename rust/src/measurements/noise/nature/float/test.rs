use crate::{measurements::make_laplace, metrics::AbsoluteDistance};

use super::*;

#[test]
fn test_make_noise_floatexpfamily() -> Fallible<()> {
    let space = (
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
    );

    assert!(
        FloatExpFamily::<1> { scale: 1.0, k: 0 }
            .make_noise(space.clone())
            .is_ok()
    );
    assert!(
        FloatExpFamily::<1> {
            scale: f64::NAN,
            k: 0
        }
        .make_noise(space.clone())
        .is_err()
    );
    assert!(
        FloatExpFamily::<2> {
            scale: 1.0,
            k: i32::MIN
        }
        .make_noise(space.clone())
        .is_ok()
    );

    assert!(
        FloatExpFamily::<2> {
            scale: 1.0,
            k: i32::MAX
        }
        .make_noise(space.clone())
        .is_ok()
    );

    Ok(())
}

#[allow(non_snake_case)]
fn sample_continuous_laplace(shift: f64, scale: f64, k: i32) -> Fallible<f64> {
    make_laplace(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<i8>::default(),
        scale,
        Some(k),
    )?
    .invoke(&shift)
}

#[test]
fn test_continuous_laplace_ignores_positive_k() -> Fallible<()> {
    assert_eq!(sample_continuous_laplace(-3.0, 0.0, 2)?, -3.0);
    assert_eq!(
        sample_continuous_laplace(-3.6522343492937, 0.0, 2)?,
        -3.6522343492937
    );
    assert_eq!(sample_continuous_laplace(3.0, 0.0, 2)?, 3.0);
    assert!(sample_continuous_laplace(4.0, 23.0, 2)?.is_finite());

    Ok(())
}

#[test]
fn test_continuous_laplace_ignores_negative_k() -> Fallible<()> {
    assert_eq!(sample_continuous_laplace(-100.23, 0.0, -2)?, -100.23);
    assert_eq!(sample_continuous_laplace(-0.1, 0.0, -2)?, -0.1);
    assert_eq!(sample_continuous_laplace(0.125, 0.0, -2)?, 0.125);
    assert!(sample_continuous_laplace(2.8954, 2.0, -2)?.is_finite());

    Ok(())
}
