use crate::transformations::Pairwise;

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

macro_rules! test_covariance_bounds {
    ($module:ident, $float:ty, $sum:ident) => {
        mod $module {
            use super::*;
            use crate::core::Domain;
            use crate::transformations::$sum;

            #[test]
            fn rejects_inverted_bounds() {
                for (bounds_0, bounds_1) in [
                    ((0., 2.), (12., 10.)),
                    ((2., 0.), (10., 12.)),
                    ((2., 0.), (12., 10.)),
                ] {
                    let err =
                        make_sized_bounded_covariance::<$sum<$float>>(5, bounds_0, bounds_1, 1)
                            .unwrap_err();
                    assert_eq!(err.variant, crate::error::ErrorVariant::MakeDomain);
                }
            }

            #[test]
            fn accepts_valid_and_degenerate_bounds() -> Fallible<()> {
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
                        let t = make_sized_bounded_covariance::<$sum<$float>>(
                            5, bounds_0, bounds_1, ddof,
                        )?;
                        assert!(t.input_domain.member(&arg)?);
                        let expected = if bounds_0.0 == bounds_0.1 || bounds_1.0 == bounds_1.1 {
                            0.
                        } else {
                            2. / (5 - ddof) as $float
                        };
                        assert_eq!(t.invoke(&arg)?, expected);
                        for d_in in [0, 1, 2, 10] {
                            let stability = t.map(&d_in)?;
                            assert!(stability.is_finite() && stability >= 0.);
                        }
                    }
                }
                Ok(())
            }

            #[test]
            fn rejects_nan_bounds() {
                for (bounds_0, bounds_1) in [
                    ((<$float>::NAN, 2.), (10., 12.)),
                    ((0., <$float>::NAN), (10., 12.)),
                    ((0., 2.), (<$float>::NAN, 12.)),
                    ((0., 2.), (10., <$float>::NAN)),
                ] {
                    assert!(
                        make_sized_bounded_covariance::<$sum<$float>>(5, bounds_0, bounds_1, 1)
                            .is_err()
                    );
                }
            }
        }
    };
}

test_covariance_bounds!(pairwise_f32, f32, Pairwise);
test_covariance_bounds!(pairwise_f64, f64, Pairwise);
test_covariance_bounds!(sequential_f32, f32, Sequential);
test_covariance_bounds!(sequential_f64, f64, Sequential);
