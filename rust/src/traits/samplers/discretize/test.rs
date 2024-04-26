use super::*;
#[test]
fn test_sample_discrete_laplace() -> Fallible<()> {
    let dgeo: f64 = sample_discrete_laplace_Z2k(0f64, 1f64, 50)?;
    println!("final: {:?}", dgeo);

    // let dgeo: f64 = f64::sample_discrete_laplace(0f64, 20f64, 14)?;
    // println!("final: {:?}", dgeo);
    Ok(())
}

#[test]
fn test_sample_discrete_laplace_pos_k() -> Fallible<()> {
    // check rounding of negative arguments
    assert_eq!(sample_discrete_laplace_Z2k(-4., 0f64, 2)?, -4.0);
    assert_eq!(sample_discrete_laplace_Z2k(-3., 0f64, 2)?, -4.0);
    assert_eq!(sample_discrete_laplace_Z2k(-2., 0f64, 2)?, -4.0);
    assert_eq!(sample_discrete_laplace_Z2k(-1., 0f64, 2)?, 0.0);
    assert_eq!(
        sample_discrete_laplace_Z2k(-3.6522343492937, 0f64, 2)?,
        -4.0
    );

    assert_eq!(sample_discrete_laplace_Z2k(0., 0f64, 2)?, 0.0);

    // check rounding of positive arguments
    assert_eq!(sample_discrete_laplace_Z2k(1., 0f64, 2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(2., 0f64, 2)?, 4.0);
    assert_eq!(sample_discrete_laplace_Z2k(3., 0f64, 2)?, 4.0);
    assert_eq!(sample_discrete_laplace_Z2k(4., 0f64, 2)?, 4.0);
    assert_eq!(sample_discrete_laplace_Z2k(3.6522343492937, 0f64, 2)?, 4.0);

    // check that noise is applied in increments of 4
    assert_eq!(sample_discrete_laplace_Z2k(4., 23f64, 2)? % 4., 0.);
    assert_eq!(sample_discrete_laplace_Z2k(4., 2f64, 2)? % 4., 0.);
    assert_eq!(sample_discrete_laplace_Z2k(4., 456e3f64, 2)? % 4., 0.);

    Ok(())
}

#[test]
fn test_sample_discrete_laplace_neg_k() -> Fallible<()> {
    assert_eq!(sample_discrete_laplace_Z2k(-100.23, 0f64, -2)?, -100.25);
    assert_eq!(sample_discrete_laplace_Z2k(-34.29, 0f64, -2)?, -34.25);
    assert_eq!(sample_discrete_laplace_Z2k(-0.1, 0f64, -2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(0., 0f64, -2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(0.1, 0f64, -2)?, 0.0);
    assert_eq!(sample_discrete_laplace_Z2k(0.125, 0f64, -2)?, 0.25);
    assert_eq!(sample_discrete_laplace_Z2k(0.13, 0f64, -2)?, 0.25);

    // check that noise is applied in increments of .25
    assert_eq!(
        sample_discrete_laplace_Z2k(2342.234532, 23f64, -2)? % 0.25,
        0.
    );
    assert_eq!(sample_discrete_laplace_Z2k(2.8954, 2f64, -2)? % 0.25, 0.);
    assert_eq!(
        sample_discrete_laplace_Z2k(834.349, 456e3f64, -2)? % 0.25,
        0.
    );

    Ok(())
}

#[test]
fn test_extreme_rational() -> Fallible<()> {
    // rationals with greater magnitude than MAX saturate to infinity
    let rat = RBig::try_from(f64::MAX).unwrap();
    assert!((rat * IBig::from(2u8)).to_f64().value().is_infinite());

    Ok(())
}

#[test]
fn test_shr() -> Fallible<()> {
    assert_eq!(shr(RBig::try_from(1.)?, 0), RBig::ONE);
    assert_eq!(shr(RBig::try_from(0.25)?, -2), RBig::ONE);
    assert_eq!(shr(RBig::try_from(1.)?, 2), RBig::try_from(0.25)?);
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
