use crate::core::*;
use crate::domains::AtomDomain;
use crate::error::ExplainUnwrap;
use crate::measures::MaxDivergence;
use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
fn test_make_chain_mt() -> Fallible<()> {
    let input_domain0 = AtomDomain::<u8>::default();
    let output_domain0 = AtomDomain::<i32>::default();
    let function0 = Function::new(|a: &u8| (a + 1) as i32);
    let input_metric0 = AbsoluteDistance::<i32>::default();
    let output_metric0 = AbsoluteDistance::<i32>::default();
    let stability_map0 = StabilityMap::new_from_constant(1);

    let transformation0 = Transformation::new(
        input_domain0,
        output_domain0,
        function0,
        input_metric0,
        output_metric0,
        stability_map0,
    )?;
    let input_domain1 = AtomDomain::<i32>::default();
    let function1 = Function::new(|a: &i32| (a + 1) as f64);
    let input_metric1 = AbsoluteDistance::<i32>::default();
    let output_measure1 = MaxDivergence::default();
    let privacy_map1 = PrivacyMap::new(|d_in: &i32| *d_in as f64 + 1.);
    let measurement1 = Measurement::new(
        input_domain1,
        function1,
        input_metric1,
        output_measure1,
        privacy_map1,
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
    let input_domain0 = AtomDomain::<u8>::default();
    let output_domain0 = AtomDomain::<i32>::default();
    let function0 = Function::new(|a: &u8| (a + 1) as i32);
    let input_metric0 = AbsoluteDistance::<i32>::default();
    let output_metric0 = AbsoluteDistance::<i32>::default();
    let stability_map0 = StabilityMap::new_from_constant(1);
    let transformation0 = Transformation::new(
        input_domain0,
        output_domain0,
        function0,
        input_metric0,
        output_metric0,
        stability_map0,
    )?;
    let input_domain1 = AtomDomain::<i32>::default();
    let output_domain1 = AtomDomain::<f64>::default();
    let function1 = Function::new(|a: &i32| (a + 1) as f64);
    let input_metric1 = AbsoluteDistance::<i32>::default();
    let output_metric1 = AbsoluteDistance::<i32>::default();
    let stability_map1 = StabilityMap::new_from_constant(1);
    let transformation1 = Transformation::new(
        input_domain1,
        output_domain1,
        function1,
        input_metric1,
        output_metric1,
        stability_map1,
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
    let input_domain0 = AtomDomain::<u8>::default();
    let function0 = Function::new(|a: &u8| (a + 1) as i32);
    let input_metric0 = AbsoluteDistance::<i32>::default();
    let output_measure0 = MaxDivergence::default();
    let privacy_map0 = PrivacyMap::new_from_constant(1.);
    let measurement0 = Measurement::new(
        input_domain0,
        function0,
        input_metric0,
        output_measure0,
        privacy_map0,
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
