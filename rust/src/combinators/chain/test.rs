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

#[test]
fn test_make_chain_tt_and_shr_preserve_result_semantics() -> Fallible<()> {
    // These errors are deliberately outside the relevant input domains.
    // The test checks executable propagation; the Verus lemma establishes
    // the separate semantic validity theorem for certified transformations.
    let inner = Transformation::new(
        AtomDomain::<i32>::new_closed((0, 10))?,
        AbsoluteDistance::<i32>::default(),
        AtomDomain::<i32>::new_closed((1, 11))?,
        AbsoluteDistance::<i32>::default(),
        Function::new_fallible(|x: &i32| {
            if *x < 0 {
                return fallible!(FailedFunction, "inner function error");
            }
            x.checked_add(1)
                .ok_or_else(|| err!(FailedFunction, "overflow"))
        }),
        StabilityMap::new_fallible(|d: &i32| {
            if *d < 0 {
                return fallible!(FailedMap, "inner map error");
            }
            d.checked_add(1)
                .ok_or_else(|| err!(FailedMap, "overflow"))
        }),
    )?;
    let outer = Transformation::new(
        AtomDomain::<i32>::new_closed((1, 11))?,
        AbsoluteDistance::<i32>::default(),
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        Function::new_fallible(|x: &i32| {
            if *x > 50 {
                return fallible!(FailedFunction, "outer function error");
            }
            x.checked_mul(2)
                .ok_or_else(|| err!(FailedFunction, "overflow"))
        }),
        StabilityMap::new_fallible(|d: &i32| {
            if *d == 9 {
                return fallible!(FailedMap, "outer map error");
            }
            d.checked_mul(2)
                .ok_or_else(|| err!(FailedMap, "overflow"))
        }),
    )?;

    let chained = make_chain_tt(&outer, &inner)?;
    let via_shr = (inner.clone() >> outer.clone())?;

    // Check success, inner error, and outer error through both entrypoints.
    for input in [-1, 0, 8, 50, 99] {
        let expected = inner.invoke(&input)
            .and_then(|value| outer.invoke(&value));
        let actual = chained.invoke(&input);
        let shr = via_shr.invoke(&input);
        // Error identity, not merely the presence of an error, is observable.
        assert_eq!(actual, expected);
        assert_eq!(shr, expected);
    }

    // Check success and both possible map-error propagation paths.
    for distance in [-1, 0, 2, 8] {
        let expected = inner.map(&distance)
            .and_then(|bound| outer.map(&bound));
        let actual = chained.map(&distance);
        let shr = via_shr.map(&distance);
        // Error identity, not merely the presence of an error, is observable.
        assert_eq!(actual, expected);
        assert_eq!(shr, expected);
    }

    // The constructor and its operator reject unequal intermediate domains.
    let incompatible = Transformation::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        Function::new(|x: &i32| *x),
        StabilityMap::new(|d: &i32| *d),
    )?;
    assert!(make_chain_tt(&incompatible, &inner).is_err());
    assert!((inner >> incompatible).is_err());
    Ok(())
}
