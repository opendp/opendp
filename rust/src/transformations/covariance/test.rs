use crate::{
    core::Domain,
    transformations::{Pairwise, Sequential},
};

use super::*;

#[test]
fn test_make_sized_bounded_covariance() -> Fallible<()> {
    let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

    let transformation_sample =
        make_sized_bounded_covariance::<Pairwise<f64>>(5, (0., 2.), (10., 12.), 1)?;
    let ret = transformation_sample.invoke(&arg)?;
    let expected = 2.5;
    assert_eq!(ret, expected);
    assert!(transformation_sample.check(&1, &(100. / 5.))?);

    let transformation_pop =
        make_sized_bounded_covariance::<Pairwise<f64>>(5, (0., 2.), (10., 12.), 0)?;
    let ret = transformation_pop.invoke(&arg)?;
    let expected = 2.0;
    assert_eq!(ret, expected);
    assert!(transformation_pop.check(&1, &(100. * 4. / 25.))?);
    Ok(())
}

#[test]
fn test_covariance_bounds_f32() -> Fallible<()> {
    for make in [
        make_sized_bounded_covariance::<Pairwise<f32>>,
        make_sized_bounded_covariance::<Sequential<f32>>,
    ] {
        for (bounds_0, bounds_1) in [
            ((0., 2.), (12., 10.)),
            ((2., 0.), (10., 12.)),
            ((2., 0.), (12., 10.)),
        ] {
            let err = make(5, bounds_0, bounds_1, 1).unwrap_err();
            assert_eq!(err.variant, crate::error::ErrorVariant::MakeDomain);
        }

        for (bounds_0, bounds_1) in [
            ((0., 2.), (10., 12.)),
            ((1., 1.), (10., 12.)),
            ((0., 2.), (11., 11.)),
            ((1., 1.), (11., 11.)),
        ] {
            let arg = vec![
                (bounds_0.0, bounds_1.0),
                (bounds_0.1, bounds_1.1),
                (1., 11.),
                (1., 11.),
                (1., 11.),
            ];
            for ddof in [0, 1] {
                let t = make(5, bounds_0, bounds_1, ddof)?;
                assert!(t.input_domain.member(&arg)?);
                let expected = if bounds_0.0 == bounds_0.1 || bounds_1.0 == bounds_1.1 {
                    0.
                } else {
                    2. / (5 - ddof) as f32
                };
                assert_eq!(t.invoke(&arg)?, expected);
                for d_in in [0, 1, 2, 10] {
                    let stability = t.map(&d_in)?;
                    assert!(stability.is_finite() && stability >= 0.);
                }
            }
        }

        for (bounds_0, bounds_1) in [
            ((f32::NAN, 2.), (10., 12.)),
            ((0., f32::NAN), (10., 12.)),
            ((0., 2.), (f32::NAN, 12.)),
            ((0., 2.), (10., f32::NAN)),
        ] {
            assert!(make(5, bounds_0, bounds_1, 1).is_err());
        }
    }
    Ok(())
}

#[test]
fn test_covariance_bounds_f64() -> Fallible<()> {
    for make in [
        make_sized_bounded_covariance::<Pairwise<f64>>,
        make_sized_bounded_covariance::<Sequential<f64>>,
    ] {
        for (bounds_0, bounds_1) in [
            ((0., 2.), (12., 10.)),
            ((2., 0.), (10., 12.)),
            ((2., 0.), (12., 10.)),
        ] {
            let err = make(5, bounds_0, bounds_1, 1).unwrap_err();
            assert_eq!(err.variant, crate::error::ErrorVariant::MakeDomain);
        }

        for (bounds_0, bounds_1) in [
            ((0., 2.), (10., 12.)),
            ((1., 1.), (10., 12.)),
            ((0., 2.), (11., 11.)),
            ((1., 1.), (11., 11.)),
        ] {
            let arg = vec![
                (bounds_0.0, bounds_1.0),
                (bounds_0.1, bounds_1.1),
                (1., 11.),
                (1., 11.),
                (1., 11.),
            ];
            for ddof in [0, 1] {
                let t = make(5, bounds_0, bounds_1, ddof)?;
                assert!(t.input_domain.member(&arg)?);
                let expected = if bounds_0.0 == bounds_0.1 || bounds_1.0 == bounds_1.1 {
                    0.
                } else {
                    2. / (5 - ddof) as f64
                };
                assert_eq!(t.invoke(&arg)?, expected);
                for d_in in [0, 1, 2, 10] {
                    let stability = t.map(&d_in)?;
                    assert!(stability.is_finite() && stability >= 0.);
                }
            }
        }

        for (bounds_0, bounds_1) in [
            ((f64::NAN, 2.), (10., 12.)),
            ((0., f64::NAN), (10., 12.)),
            ((0., 2.), (f64::NAN, 12.)),
            ((0., 2.), (10., f64::NAN)),
        ] {
            assert!(make(5, bounds_0, bounds_1, 1).is_err());
        }
    }
    Ok(())
}
