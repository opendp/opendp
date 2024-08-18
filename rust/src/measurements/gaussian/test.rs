use dashu::{ibig, rbig};

use super::*;
use crate::{
    domains::{AtomDomain, VectorDomain}, measures::ZeroConcentratedDivergence, metrics::{AbsoluteDistance, L2Distance}
};

#[test]
fn test_vector_bigint_gaussian_big_scale() -> Fallible<()> {
    let meas = make_noise::<_, _, _, ZeroConcentratedDivergence>(
        VectorDomain::default(), L2Distance::default(),
        ZExpFamily {
            scale: rbig!(23948285282902934157)
        },
    )?;
    println!("{:?}", meas.invoke(&vec![ibig!(0)])?);
    assert!(meas.check(&rbig!(1), &1e30f64)?);
    Ok(())
}

#[test]
fn test_vector_bigint_gaussian_zero_scale() -> Fallible<()> {
    let meas = make_noise::<_, _, _, ZeroConcentratedDivergence>(
        VectorDomain::default(), L2Distance::default(),
        ZExpFamily {
            scale: rbig!(0),
        }
    )?;
    assert_eq!(meas.invoke(&vec![ibig!(0)])?, vec![ibig!(0)]);
    assert_eq!(meas.map(&rbig!(0))?, 0.);
    assert_eq!(meas.map(&rbig!(1))?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_make_scalar_float_gaussian() -> Fallible<()> {
    let measurement = make_gaussian::<_, _, ZeroConcentratedDivergence>(
        AtomDomain::default(),
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
        VectorDomain::new(AtomDomain::default()),
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
        AtomDomain::default(),
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
        AtomDomain::default(),
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
        AtomDomain::default(),
        AbsoluteDistance::<f64>::default(),
        f64::MAX,
        None,
    )?;
    println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

    Ok(())
}
