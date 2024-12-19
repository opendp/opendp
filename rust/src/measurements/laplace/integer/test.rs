use super::*;
use crate::{domains::AtomDomain, metrics::AbsoluteDistance};

// there is a distributional test in the accuracy module

#[test]
fn test_discrete_laplace_cks20() -> Fallible<()> {
    let meas =
        make_scalar_integer_laplace(AtomDomain::default(), AbsoluteDistance::default(), 1e30f64)?;
    println!("{:?}", meas.invoke(&0)?);
    assert!(meas.check(&1, &1e30f64)?);
    Ok(())
}

#[test]
fn test_discrete_laplace_cks20_zero_scale() -> Fallible<()> {
    let meas = make_scalar_integer_laplace(AtomDomain::default(), AbsoluteDistance::default(), 0.)?;
    assert_eq!(meas.invoke(&0)?, 0);
    assert_eq!(meas.map(&0)?, 0.);
    assert_eq!(meas.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_discrete_laplace_cks20_max_scale() -> Fallible<()> {
    let meas =
        make_scalar_integer_laplace(AtomDomain::default(), AbsoluteDistance::default(), f64::MAX)?;
    println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

    Ok(())
}
