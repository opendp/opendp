use super::*;
use crate::{domains::AtomDomain, measures::ZeroConcentratedDivergence};

// there is a distributional test in the accuracy module

#[test]
fn test_make_scalar_integer_gaussian() -> Fallible<()> {
    let meas = make_scalar_integer_gaussian::<_, ZeroConcentratedDivergence, f32>(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        1e30f64,
    )?;
    println!("{:?}", meas.invoke(&0)?);
    assert!(meas.check(&1., &1e30f64.recip().powi(2))?);
    Ok(())
}

#[test]
fn test_make_scalar_integer_gaussian_zero_scale() -> Fallible<()> {
    let meas = make_scalar_integer_gaussian::<_, ZeroConcentratedDivergence, i32>(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        0.,
    )?;
    assert_eq!(meas.invoke(&0)?, 0);
    assert_eq!(meas.map(&0)?, 0.);
    assert_eq!(meas.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_make_scalar_integer_gaussian_max_scale() -> Fallible<()> {
    let meas = make_scalar_integer_gaussian::<_, ZeroConcentratedDivergence, f64>(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        f64::MAX,
    )?;
    println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

    Ok(())
}
