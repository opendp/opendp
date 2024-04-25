use crate::{
    domains::{AtomDomain, VectorDomain},
    metrics::AbsoluteDistance,
};

use super::*;

#[test]
fn test_make_geometric_bounded() -> Fallible<()> {
    let measurement = make_scalar_geometric(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        10.0,
        Some((200, 210)),
    )?;
    let arg = 205;
    let _ret = measurement.invoke(&arg)?;
    println!("{:?}", _ret);

    assert!(measurement.check(&1, &0.5)?);
    Ok(())
}

#[test]
fn test_make_vector_geometric_bounded() -> Fallible<()> {
    let measurement = make_vector_geometric(
        VectorDomain::new(AtomDomain::default()),
        Default::default(),
        10.0,
        Some((200, 210)),
    )?;
    let arg = vec![1, 2, 3, 4];
    let _ret = measurement.invoke(&arg)?;
    println!("{:?}", _ret);

    assert!(measurement.check(&1, &0.5)?);
    Ok(())
}

#[test]
fn test_make_geometric_mechanism() -> Fallible<()> {
    let measurement = make_scalar_geometric(AtomDomain::default(), Default::default(), 10.0, None)?;
    let arg = 205;
    let _ret = measurement.invoke(&arg)?;
    println!("{:?}", _ret);

    assert!(measurement.check(&1, &0.5)?);
    Ok(())
}

#[test]
fn test_make_vector_geometric_mechanism() -> Fallible<()> {
    let measurement = make_vector_geometric(
        VectorDomain::new(AtomDomain::default()),
        Default::default(),
        10.0,
        None,
    )?;
    let arg = vec![1, 2, 3, 4];
    let _ret = measurement.invoke(&arg)?;
    println!("{:?}", _ret);

    assert!(measurement.check(&1, &0.5)?);
    Ok(())
}
