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
