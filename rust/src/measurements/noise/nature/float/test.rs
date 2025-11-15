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
        .is_err()
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

#[test]
fn test_then_deintegerize_vec() -> Fallible<()> {
    // rationals with greater magnitude than MAX saturate to infinity
    let q = RBig::try_from(f64::MAX).unwrap();
    assert!((q * IBig::from(2u8)).to_f64().value().is_infinite());

    assert!(then_deintegerize_vec::<f64>(i32::MIN).is_err());
    assert!(then_deintegerize_vec::<f64>(i32::MAX).is_ok());
    assert!(then_deintegerize_vec::<f64>(0).is_ok());
    Ok(())
}

#[allow(non_snake_case)]
fn sample_dlap_Z2K(shift: f64, scale: f64, k: i32) -> Fallible<f64> {
    make_laplace(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<i8>::default(),
        scale,
        Some(k),
    )?
    .invoke(&shift)
}

#[test]
fn test_make_float_to_bigint_pos_k() -> Fallible<()> {
    // check rounding of negative arguments
    assert_eq!(sample_dlap_Z2K(-4., 0.0, 2)?, -4.);
    assert_eq!(sample_dlap_Z2K(-3.0, 0.0, 2)?, -4.0);
    assert_eq!(sample_dlap_Z2K(-2.0, 0.0, 2)?, 0.0);
    assert_eq!(sample_dlap_Z2K(-1.0, 0.0, 2)?, 0.0);
    assert_eq!(sample_dlap_Z2K(-3.6522343492937, 0.0, 2)?, -4.0);

    assert_eq!(sample_dlap_Z2K(0.0, 0.0, 2)?, 0.0);

    // check rounding of positive arguments
    assert_eq!(sample_dlap_Z2K(1.0, 0.0, 2)?, 0.0);
    assert_eq!(sample_dlap_Z2K(2.0, 0.0, 2)?, 4.0);
    assert_eq!(sample_dlap_Z2K(3.0, 0.0, 2)?, 4.0);
    assert_eq!(sample_dlap_Z2K(4.0, 0.0, 2)?, 4.0);
    assert_eq!(sample_dlap_Z2K(3.6522343492937, 0.0, 2)?, 4.0);

    // check that noise is applied in increments of 4
    assert_eq!(sample_dlap_Z2K(4.0, 23.0, 2)? % 4.0, 0.0);
    assert_eq!(sample_dlap_Z2K(4.0, 2.0, 2)? % 4.0, 0.0);
    assert_eq!(sample_dlap_Z2K(4.0, 456e3f64, 2)? % 4.0, 0.0);

    Ok(())
}

#[test]
fn test_make_float_to_bigint_neg_k() -> Fallible<()> {
    assert_eq!(sample_dlap_Z2K(-100.23, 0.0, -2)?, -100.25);
    assert_eq!(sample_dlap_Z2K(-34.29, 0.0, -2)?, -34.25);
    assert_eq!(sample_dlap_Z2K(-0.1, 0.0, -2)?, 0.0);
    assert_eq!(sample_dlap_Z2K(0.0, 0.0, -2)?, 0.0);
    assert_eq!(sample_dlap_Z2K(0.1, 0.0, -2)?, 0.0);
    assert_eq!(sample_dlap_Z2K(0.125, 0.0, -2)?, 0.25);
    assert_eq!(sample_dlap_Z2K(0.13, 0.0, -2)?, 0.25);

    // check that noise is applied in increments of .25
    assert_eq!(sample_dlap_Z2K(2342.234532, 23.0, -2)? % 0.25, 0.0);
    assert_eq!(sample_dlap_Z2K(2.8954, 2.0, -2)? % 0.25, 0.0);
    assert_eq!(sample_dlap_Z2K(834.349, 456e3f64, -2)? % 0.25, 0.0);

    Ok(())
}
