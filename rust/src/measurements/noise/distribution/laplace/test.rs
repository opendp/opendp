use super::*;
use crate::{
    domains::{AtomDomain, VectorDomain},
    metrics::{AbsoluteDistance, L1Distance, SymmetricDistance},
    transformations::make_mean,
};
use num::{One, Zero};

// there is a distributional test in the accuracy module

#[test]
fn test_all() -> Fallible<()> {
    macro_rules! test_laplace_with_ty {
        ($($ty:ty),+) => {$(
            let meas = make_laplace(AtomDomain::<$ty>::default(), Default::default(), 1., None)?;
            meas.invoke(&<$ty>::zero())?;
            meas.map(&<$ty>::one())?;

            let meas = make_laplace(VectorDomain::new(AtomDomain::<$ty>::default()), Default::default(), 1., None)?;
            meas.invoke(&vec![<$ty>::zero()])?;
            meas.map(&<$ty>::one())?;
        )+}
    }
    test_laplace_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64);
    Ok(())
}

#[test]
fn test_chain_laplace() -> Fallible<()> {
    let chain = (make_mean(
        VectorDomain::new(AtomDomain::new_closed((10., 12.))?).with_size(3),
        SymmetricDistance,
    )? >> make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::default(),
        1.0,
        None,
    )?)?;
    let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
    Ok(())
}

#[test]
fn test_big_laplace() -> Fallible<()> {
    let chain = make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::<i32>::default(),
        f64::MAX,
        None,
    )?;
    println!("{:?}", chain.invoke(&f64::MAX)?);
    Ok(())
}

#[test]
fn test_make_laplace_mechanism() -> Fallible<()> {
    let measurement = make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<f64>::default(),
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
    let measurement = make_laplace::<_, _, MaxDivergence>(
        VectorDomain::new(AtomDomain::<f64>::default()),
        L1Distance::<f64>::default(),
        1.0,
        None,
    )?;
    let arg = vec![1.0, 2.0, 3.0];
    let _ret = measurement.invoke(&arg)?;

    assert!(measurement.check(&1., &1.)?);
    Ok(())
}

#[test]
fn test_discrete_laplace_cks20() -> Fallible<()> {
    let meas = make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<i64>::default(),
        AbsoluteDistance::<i32>::default(),
        1e30f64,
        None,
    )?;
    println!("{:?}", meas.invoke(&0)?);
    assert!(meas.check(&1, &1e30f64)?);
    Ok(())
}

#[test]
fn test_discrete_laplace_cks20_zero_scale() -> Fallible<()> {
    let meas = make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<u8>::default(),
        AbsoluteDistance::default(),
        0.,
        None,
    )?;
    assert_eq!(meas.invoke(&0)?, 0);
    assert_eq!(meas.map(&0)?, 0.);
    assert_eq!(meas.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_discrete_laplace_cks20_max_scale() -> Fallible<()> {
    let meas = make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<u16>::default(),
        AbsoluteDistance::<i32>::default(),
        f64::MAX,
        None,
    )?;
    println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

    Ok(())
}

#[test]
fn test_scalar_integer_laplace_zero_scale() -> Fallible<()> {
    let meas = make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<i16>::default(),
        AbsoluteDistance::default(),
        0.,
        None,
    )?;
    assert_eq!(meas.invoke(&0)?, 0);
    assert_eq!(meas.map(&0)?, 0.);
    assert_eq!(meas.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_scalar_integer_laplace_max_scale() -> Fallible<()> {
    let meas = make_laplace::<_, _, MaxDivergence>(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        f64::MAX,
        None,
    )?;
    println!("{:?} {:?}", meas.invoke(&0)?, i32::MAX);

    Ok(())
}
