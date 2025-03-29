use crate::{measurements::make_laplace, metrics::AbsoluteDistance};

use super::*;

#[allow(non_snake_case)]
fn sample_discrete_laplace_Z2k(shift: f64, scale: f64, k: i32) -> Fallible<f64> {
    make_laplace(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<i8>::default(),
        scale,
        Some(k),
    )?
    .invoke(&shift)
}

#[test]
fn test_sample_discrete_laplace_pos_k() -> Fallible<()> {
    // check rounding of negative arguments
    assert_eq!(sample_discrete_laplace_Z2k(-4., 0.0, 2)?, -4.);
    assert_eq!(sample_discrete_laplace_Z2k(-3.0, 0.0, 2)?, -4.0);
    assert_eq!(sample_discrete_laplace_Z2k(-2.0, 0.0, 2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(-1.0, 0.0, 2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(-3.6522343492937, 0.0, 2)?, -4.0);

    assert_eq!(sample_discrete_laplace_Z2k(0.0, 0.0, 2)?, 0.0);

    // check rounding of positive arguments
    assert_eq!(sample_discrete_laplace_Z2k(1.0, 0.0, 2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(2.0, 0.0, 2)?, 4.0);
    assert_eq!(sample_discrete_laplace_Z2k(3.0, 0.0, 2)?, 4.0);
    assert_eq!(sample_discrete_laplace_Z2k(4.0, 0.0, 2)?, 4.0);
    assert_eq!(sample_discrete_laplace_Z2k(3.6522343492937, 0.0, 2)?, 4.0);

    // check that noise is applied in increments of 4
    assert_eq!(sample_discrete_laplace_Z2k(4.0, 23.0, 2)? % 4.0, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(4.0, 2.0, 2)? % 4.0, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(4.0, 456e3f64, 2)? % 4.0, 0.0);

    Ok(())
}

#[test]
fn test_sample_discrete_laplace_neg_k() -> Fallible<()> {
    assert_eq!(sample_discrete_laplace_Z2k(-100.23, 0.0, -2)?, -100.25);
    assert_eq!(sample_discrete_laplace_Z2k(-34.29, 0.0, -2)?, -34.25);
    assert_eq!(sample_discrete_laplace_Z2k(-0.1, 0.0, -2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(0.0, 0.0, -2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(0.1, 0.0, -2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(0.125, 0.0, -2)?, 0.25);
    assert_eq!(sample_discrete_laplace_Z2k(0.13, 0.0, -2)?, 0.25);

    // check that noise is applied in increments of .25
    assert_eq!(
        sample_discrete_laplace_Z2k(2342.234532, 23.0, -2)? % 0.25,
        0.0
    );
    assert_eq!(sample_discrete_laplace_Z2k(2.8954, 2.0, -2)? % 0.25, 0.0);
    assert_eq!(
        sample_discrete_laplace_Z2k(834.349, 456e3f64, -2)? % 0.25,
        0.0
    );

    Ok(())
}
