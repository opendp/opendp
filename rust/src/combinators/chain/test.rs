use crate::core::*;
use crate::domains::AtomDomain;
use crate::error::ExplainUnwrap;
use crate::measures::MaxDivergence;
use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
fn test_make_chain_mt() -> Fallible<()> {
    let transformation0 = Transformation::new(
        AtomDomain::<u8>::default(),
        AbsoluteDistance::<i32>::default(),
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        Function::new(|a: &u8| (a + 1) as i32),
        StabilityMap::new_from_constant(1),
    )?;
    let measurement1 = Measurement::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        MaxDivergence,
        Function::new(|a: &i32| (a + 1) as f64),
        PrivacyMap::new(|d_in: &i32| *d_in as f64 + 1.),
    )?;
    let chain = make_chain_mt(&measurement1, &transformation0)?;

    let arg = 99_u8;
    let ret = chain.invoke(&arg).unwrap_test();
    assert_eq!(ret, 101.0);

    let d_in = 99_i32;
    let d_out = chain.map(&d_in).unwrap_test();
    assert_eq!(d_out, 100.);

    Ok(())
}

#[test]
fn test_make_chain_tt() -> Fallible<()> {
    let transformation0 = Transformation::new(
        AtomDomain::<u8>::default(),
        AbsoluteDistance::<i32>::default(),
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        Function::new(|a: &u8| (a + 1) as i32),
        StabilityMap::new_from_constant(1),
    )?;
    let transformation1 = Transformation::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<i32>::default(),
        Function::new(|a: &i32| (a + 1) as f64),
        StabilityMap::new_from_constant(1),
    )?;
    let chain = make_chain_tt(&transformation1, &transformation0).unwrap_test();

    let arg = 99_u8;
    let ret = chain.invoke(&arg).unwrap_test();
    assert_eq!(ret, 101.0);

    let d_in = 99_i32;
    let d_out = chain.map(&d_in).unwrap_test();
    assert_eq!(d_out, 99);

    Ok(())
}

#[test]
fn test_make_chain_pm() -> Fallible<()> {
    let measurement0 = Measurement::new(
        AtomDomain::<u8>::default(),
        AbsoluteDistance::<i32>::default(),
        MaxDivergence,
        Function::new(|a: &u8| (a + 1) as i32),
        PrivacyMap::new_from_constant(1.),
    )?;
    let function1 = Function::new(|a: &i32| (a + 1) as f64);
    let chain = make_chain_pm(&function1, &measurement0)?;

    let arg = 99_u8;
    let ret = chain.invoke(&arg)?;
    assert_eq!(ret, 101.0);

    let d_in = 99_i32;
    let d_out = chain.map(&d_in)?;
    assert_eq!(d_out, 99.);

    Ok(())
}
