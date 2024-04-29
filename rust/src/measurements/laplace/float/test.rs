use super::*;
use crate::{metrics::SymmetricDistance, transformations::make_mean};

#[test]
fn test_chain_laplace() -> Fallible<()> {
    let chain = (make_mean(
        VectorDomain::new(AtomDomain::new_closed((10., 12.))?).with_size(3),
        SymmetricDistance::default(),
    )? >> make_scalar_float_laplace(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        1.0,
        None,
    )?)?;
    let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
    Ok(())
}

#[test]
fn test_big_laplace() -> Fallible<()> {
    let chain = make_scalar_float_laplace(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        f64::MAX,
        None,
    )?;
    println!("{:?}", chain.invoke(&f64::MAX)?);
    Ok(())
}

#[test]
fn test_make_laplace_mechanism() -> Fallible<()> {
    let measurement = make_scalar_float_laplace(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        1.0,
        None,
    )?;
    let _ret = measurement.invoke(&0.0)?;

    assert!(measurement.check(&1., &1.)?);
    Ok(())
}

#[test]
fn test_make_vector_laplace_mechanism() -> Fallible<()> {
    let measurement = make_vector_float_laplace(
        VectorDomain::new(AtomDomain::default()),
        L1Distance::default(),
        1.0,
        None,
    )?;
    let arg = vec![1.0, 2.0, 3.0];
    let _ret = measurement.invoke(&arg)?;

    assert!(measurement.check(&1., &1.)?);
    Ok(())
}
