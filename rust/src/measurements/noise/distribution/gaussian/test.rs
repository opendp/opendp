use dashu::{ibig, integer::IBig, rbig};

use super::*;
use crate::{
    domains::{AtomDomain, VectorDomain},
    measures::ZeroConcentratedDivergence,
    metrics::{AbsoluteDistance, L2Distance},
};
use num::{One, Zero};

#[test]
fn test_all() -> Fallible<()> {
    macro_rules! test_gaussian_with_ty {
        ($($ty:ty),+) => {$(
            let meas = make_gaussian(AtomDomain::<$ty>::new_non_nan(), AbsoluteDistance::<$ty>::default(), 1., None)?;
            meas.invoke(&<$ty>::zero())?;
            meas.map(&<$ty>::one())?;

            let meas = make_gaussian(VectorDomain::new(AtomDomain::<$ty>::new_non_nan()), L2Distance::<$ty>::default(), 1., None)?;
            meas.invoke(&vec![<$ty>::zero()])?;
            meas.map(&<$ty>::one())?;
        )+}
    }
    test_gaussian_with_ty!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64
    );
    Ok(())
}

#[test]
fn test_vector_bigint_gaussian_big_scale() -> Fallible<()> {
    let space = (
        VectorDomain::new(AtomDomain::<IBig>::default()),
        L2Distance::default(),
    );
    let distribution = ZExpFamily {
        scale: rbig!(23948285282902934157),
    };

    let meas = distribution.make_noise(space)?;
    println!("{:?}", meas.invoke(&vec![ibig!(0)])?);
    assert!(meas.check(&rbig!(1), &1e30f64)?);
    Ok(())
}

#[test]
fn test_vector_bigint_gaussian_zero_scale() -> Fallible<()> {
    let domain = VectorDomain::<AtomDomain<IBig>>::default();
    let metric = L2Distance::default();
    let distribution = ZExpFamily { scale: rbig!(0) };

    let meas = distribution.make_noise((domain, metric))?;
    assert_eq!(meas.invoke(&vec![ibig!(0)])?, vec![ibig!(0)]);
    assert_eq!(meas.map(&rbig!(0))?, 0.);
    assert_eq!(meas.map(&rbig!(1))?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_make_scalar_float_gaussian() -> Fallible<()> {
    let measurement = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::default(),
        1.0f64,
        None,
    )?;
    let arg = 0.0;
    let _ret = measurement.invoke(&arg)?;

    assert!(measurement.check(&0.1, &0.0050000001)?);
    Ok(())
}

#[test]
fn test_make_vector_float_gaussian() -> Fallible<()> {
    let measurement = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        VectorDomain::new(AtomDomain::<f64>::new_non_nan()),
        L2Distance::default(),
        1.0f64,
        None,
    )?;
    let arg = vec![0.0, 1.0];
    let _ret = measurement.invoke(&arg)?;

    assert!(measurement.map(&0.1)? <= 0.0050000001);
    Ok(())
}
// there is a distributional test in the accuracy module

#[test]
fn test_make_scalar_integer_gaussian() -> Fallible<()> {
    let meas = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<f32>::default(),
        1e30f64,
        None,
    )?;
    println!("{:?}", meas.invoke(&0)?);
    assert!(meas.check(&1., &1e30f64.recip().powi(2))?);
    Ok(())
}

#[test]
fn test_make_scalar_integer_gaussian_zero_scale() -> Fallible<()> {
    let meas = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        AtomDomain::<i8>::default(),
        AbsoluteDistance::<i32>::default(),
        0.,
        None,
    )?;
    assert_eq!(meas.invoke(&0)?, 0);
    assert_eq!(meas.map(&0)?, 0.);
    assert_eq!(meas.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_make_scalar_integer_gaussian_max_scale() -> Fallible<()> {
    let meas = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<f64>::default(),
        f64::MAX,
        None,
    )?;
    println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

    Ok(())
}
