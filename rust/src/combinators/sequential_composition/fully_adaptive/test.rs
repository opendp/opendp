use std::any::Any;

use crate::{
    combinators::make_adaptive_composition,
    domains::AtomDomain,
    measurements::make_randomized_response_bool,
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
    metrics::DiscreteDistance,
};

use super::*;

#[test]
fn test_privacy_odometer() -> Fallible<()> {
    // construct concurrent compositor IM
    let root = make_fully_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::default(),
        DiscreteDistance::default(),
        MaxDivergence::default(),
        None,
    )?;

    // pass dataset in and receive a queryable
    let mut odometer = root.invoke(&true)?;

    let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
    let rr_query = make_randomized_response_bool(0.5, false)?;

    // pass queries into the odometer queryable
    let _answer1: bool = odometer.invoke_poly(rr_poly_query.clone())?;
    let _answer2: bool = odometer.invoke_poly(rr_poly_query.clone())?;

    // pass a concurrent composition compositor into the original CC compositor
    // This compositor expects all outputs are in AtomDomain<bool>
    let cc_query_3 = make_adaptive_composition::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance::default(),
        MaxDivergence::default(),
        1,
        vec![0.1, 0.1],
    )?
    .into_poly();

    // submitting a CC query. This CC compositor is concretely-typed
    let mut answer3 = odometer.invoke_poly::<Queryable<_, bool>>(cc_query_3)?;

    // submitting a RR query to child CC compositor with concrete types
    let _answer3_1: bool = answer3.eval(&rr_query)?;

    // submitting a second RR query to child CC compositor with concrete types
    let _answer3_2: bool = answer3.eval(&rr_query)?;

    // pass a concurrent composition compositor into the original CC compositor
    // This compositor expects all outputs are Boxed and type-erased
    let cc_query_4 = make_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::<bool>::default(),
        DiscreteDistance::default(),
        MaxDivergence::default(),
        1,
        vec![0.2, 0.3],
    )?
    .into_poly();

    // submitting a second CC query to root CC compositor with type erasure
    let mut answer4 = odometer.invoke_poly::<Queryable<_, Box<dyn Any>>>(cc_query_4)?;

    // submitting a RR query to child CC compositor
    let _answer4_1: bool = answer4.eval_poly(&rr_poly_query)?;
    let _answer4_2: bool = answer4.eval_poly(&rr_poly_query)?;

    let total_usage = odometer.privacy_loss(1)?;
    println!("total usage: {:?}", total_usage);

    Ok(())
}

#[test]
fn test_fully_adaptive_interactive_postprocessing() -> Fallible<()> {
    let m_query = (Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        Function::new_fallible(|&arg: &bool| Queryable::new_external(move |_: &()| Ok(!arg))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )? >> Function::<Queryable<(), bool>, bool>::new_fallible(|qbl: &_| {
        qbl.clone().eval(&())
    }))?;
    let m_odo = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        None,
    )?;

    let mut qbl = m_odo.invoke(&false)?;
    assert!(qbl.invoke(m_query)?);
    assert_eq!(qbl.privacy_loss(1)?, (1.0, 1e-7));
    Ok(())
}

#[test]
fn test_fully_adaptive_constructor_d_in() -> Fallible<()> {
    let odometer = |d_in| {
        make_fully_adaptive_composition::<_, _, _, bool>(
            AtomDomain::default(),
            DiscreteDistance,
            MaxDivergence,
            d_in,
        )
    };
    let m_rr = make_randomized_response_bool(0.75, false)?;
    let d_mid = m_rr.map(&1)?;

    let mut qbl = odometer(Some(1))?.invoke(&true)?;
    qbl.invoke(m_rr.clone())?;
    assert_eq!(qbl.privacy_loss(1)?, d_mid);
    assert!(qbl.privacy_loss(2).is_err());

    let mut qbl = odometer(None)?.invoke(&true)?;
    qbl.invoke(m_rr)?;
    assert_eq!(qbl.privacy_loss(1)?, d_mid);
    assert!(qbl.privacy_loss(2).is_ok());
    Ok(())
}

#[test]
fn test_fully_adaptive_failed_postprocessing_counts_privacy_loss() -> Fallible<()> {
    let rr = make_randomized_response_bool(0.75, false)?;
    let budget = rr.map(&1)?;
    let failing = (rr.clone()
        >> Function::new_fallible(|_: &bool| {
            make_randomized_response_bool(-1.0, false)?;
            Ok(false)
        }))?;
    let mut qbl = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        None,
    )?
    .invoke(&true)?;

    assert!(qbl.invoke(failing).is_err());
    assert_eq!(qbl.privacy_loss(1)?, budget);
    assert!(qbl.invoke(rr).is_ok());
    assert_eq!(
        qbl.privacy_loss(1)?,
        MaxDivergence.compose(vec![budget; 2])?
    );
    Ok(())
}

#[test]
fn test_fully_adaptive_pre_invocation_rejection_does_not_count() -> Fallible<()> {
    let measurement = |domain| {
        Measurement::new(
            domain,
            DiscreteDistance,
            MaxDivergence,
            Function::new(|_: &i32| 0_i32),
            PrivacyMap::new(|_| 1.0),
        )
    };
    let mut qbl = make_fully_adaptive_composition(
        AtomDomain::<i32>::default(),
        DiscreteDistance,
        MaxDivergence,
        None,
    )?
    .invoke(&0)?;
    assert!(
        qbl.invoke(measurement(AtomDomain::new_closed((0, 1))?)?)
            .is_err()
    );
    assert_eq!(qbl.privacy_loss(1)?, 0.0);
    assert_eq!(qbl.invoke(measurement(AtomDomain::default())?)?, 0);
    assert_eq!(qbl.privacy_loss(1)?, 1.0);
    Ok(())
}

#[test]
fn test_fully_adaptive_failure_retires_previous_sequential_child() -> Fallible<()> {
    let child = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        Function::new_fallible(|_: &bool| Queryable::new_external(|_: &()| Ok(true))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )?;
    let failing = (child.clone()
        >> Function::new_fallible(|_: &Queryable<(), bool>| {
            fallible!(FailedFunction, "postprocessing failed") as Fallible<Queryable<(), bool>>
        }))?;
    let mut qbl = make_fully_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        None,
    )?
    .invoke(&true)?;

    let mut previous: Queryable<(), bool> = qbl.invoke_poly(child.into_poly())?;
    assert!(previous.eval(&())?);
    assert!(qbl.invoke(failing.into_poly()).is_err());
    assert!(previous.eval(&()).is_err());
    assert_eq!(
        qbl.privacy_loss(1)?,
        Approximate(ZeroConcentratedDivergence).compose(vec![(1.0, 1e-7); 2])?
    );
    Ok(())
}

#[test]
fn test_fully_adaptive_failed_postprocessing_child_keeps_sequential_wrapper() -> Fallible<()> {
    let escaped = Rc::new(RefCell::new(None));
    let child = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        Function::new_fallible(|_: &bool| Queryable::new_external(|_: &()| Ok(true))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )?;
    let failing = (child.clone()
        >> Function::new_fallible(enclose!(escaped, move |qbl: &Queryable<(), bool>| {
            *escaped.borrow_mut() = Some(qbl.clone());
            fallible!(FailedFunction, "postprocessing failed") as Fallible<Queryable<(), bool>>
        })))?;
    let mut qbl = make_fully_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        None,
    )?
    .invoke(&true)?;

    assert!(qbl.invoke(failing.into_poly()).is_err());
    let mut current: Queryable<(), bool> = qbl.invoke_poly(child.into_poly())?;
    assert!(current.eval(&())?);
    assert!(escaped.borrow_mut().as_mut().unwrap().eval(&()).is_err());
    Ok(())
}

#[test]
fn test_fully_adaptive_sequential_child_accepts_queries() -> Fallible<()> {
    let child = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        Function::new_fallible(|_: &bool| Queryable::new_external(|_: &()| Ok(true))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )?;
    let mut qbl = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        None,
    )?
    .invoke(&true)?;
    assert!(qbl.invoke(child)?.eval(&())?);
    Ok(())
}
