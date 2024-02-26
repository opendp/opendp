use crate::{measurements::make_laplace, metrics::AbsoluteDistance};

use super::*;

#[test]
fn test_extreme_rational() -> Fallible<()> {
    // rationals with greater magnitude than MAX saturate to infinity
    let rat = RBig::try_from(f64::MAX).unwrap();
    assert!((rat * IBig::from(2u8)).to_f64().value().is_infinite());

    Ok(())
}

#[test]
fn test_shr() -> Fallible<()> {
    assert_eq!(x_div_2k(RBig::try_from(1.)?, 0), RBig::ONE);
    assert_eq!(x_div_2k(RBig::try_from(0.25)?, -2), RBig::ONE);
    assert_eq!(x_div_2k(RBig::try_from(1.)?, 2), RBig::try_from(0.25)?);
    Ok(())
}

#[test]
fn test_find_nearest_multiple_of_2k() -> Fallible<()> {
    assert_eq!(
        find_nearest_multiple_of_2k(RBig::try_from(-2.25)?, 0),
        IBig::from(-2)
    );
    assert_eq!(
        find_nearest_multiple_of_2k(RBig::try_from(2.25)?, -1),
        IBig::from(5)
    );
    assert_eq!(
        find_nearest_multiple_of_2k(RBig::try_from(-2.25)?, -1),
        IBig::from(-5)
    );
    Ok(())
}

#[allow(non_snake_case)]
fn sample_discrete_laplace_Z2k(shift: f64, scale: f64, k: i32) -> Fallible<f64> {
    make_laplace(
        AtomDomain::<f64>::default(),
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
    assert_eq!(sample_discrete_laplace_Z2k(-2.0, 0.0, 2)?, -4.0);
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
