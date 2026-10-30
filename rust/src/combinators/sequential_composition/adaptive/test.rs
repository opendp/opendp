use std::any::Any;

use super::*;
use crate::{
    domains::AtomDomain,
    measurements::make_randomized_response_bool,
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
    metrics::DiscreteDistance,
};

#[test]
fn test_sequential_composition() -> Fallible<()> {
    // construct sequential compositor IM
    let root = make_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::default(),
        DiscreteDistance,
        MaxDivergence,
        1,
        vec![0.1, 0.1, 0.3, 0.5],
    )?;

    // pass dataset in and receive a queryable
    let mut queryable = root.invoke(&true)?;

    // construct the leaf-node queries:
    let rr_poly_query = make_randomized_response_bool(0.5, false)?.into_poly();
    let rr_query = make_randomized_response_bool(0.5, false)?;

    // pass queries into the SC queryable
    println!(
        "the sequential compositor can be evaluated with rr poly queries, and the answer is downcast to bool"
    );
    let _answer1: bool = queryable.eval_poly(&rr_poly_query)?;
    let _answer2: bool = queryable.eval_poly(&rr_poly_query)?;

    println!(
        "\nbuild a sequential composition IM and then convert to poly, so that it can be passed to the root queryable"
    );
    // pass a sequential composition compositor into the original SC compositor
    // This compositor expects all outputs are concretely-typed (bool)
    let sc_query_3 = make_adaptive_composition::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        1,
        vec![0.1, 0.1],
    )?
    .into_poly();

    // both approaches are valid
    println!("\ncreate the sequential composition queryable as a child of the root queryable");
    let mut answer3a = queryable.eval_poly::<Queryable<_, bool>>(&sc_query_3)?;

    println!("\npass an RR query to the child sequential compositor queryable");
    let _answer3a_1: bool = answer3a.eval(&rr_query)?;

    println!("\npass a second RR query to the child sequential compositor queryable");
    let _answer3a_2: bool = answer3a.eval(&rr_query)?;

    // pass a sequential composition compositor into the original SC compositor
    // This compositor expects all outputs are in PolyDomain, but operates over dyn domains
    println!("\nbuild a dyn sequential composition IM and then convert to poly");
    let sc_query_4 = make_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        1,
        vec![0.2, 0.3],
    )?
    .into_poly();

    println!(
        "\ncreate the poly sequential composition queryable as a child of the root queryable, and downcast the queryable itself"
    );
    let mut answer4 = queryable.eval_poly::<Queryable<_, Box<dyn Any>>>(&sc_query_4)?;

    println!("\nsend a dyn query");
    let _answer4_1: bool = answer4.eval_poly(&rr_poly_query)?;

    println!("\nsend another dyn query");
    let _answer4_2: bool = answer4.eval_poly(&rr_poly_query)?;

    println!(
        "\ncan no longer send queries to the first compositor child, because a second query has been passed to its parent"
    );
    assert!(answer3a.eval(&rr_query).is_err());

    Ok(())
}

#[test]
fn test_adaptive_interactive_postprocessing() -> Fallible<()> {
    let mc = make_adaptive_composition::<_, _, _, bool>(
        AtomDomain::<bool>::default().clone(),
        DiscreteDistance.clone(),
        Approximate(ZeroConcentratedDivergence).clone(),
        1,
        vec![(1.0, 1e-7)],
    )?;

    let mut qbl = mc.invoke(&false)?;
    let m1 = (Measurement::new(
        AtomDomain::<bool>::default().clone(),
        DiscreteDistance.clone(),
        Approximate(ZeroConcentratedDivergence).clone(),
        Function::new_fallible(|&arg: &bool| Queryable::new_external(move |_: &()| Ok(arg))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )? >> Function::<Queryable<(), bool>, bool>::new_fallible(|qbl: &_| {
        qbl.clone().eval(&())
    }))?;

    assert!(!qbl.eval(&m1)?);
    Ok(())
}

#[test]
fn test_adaptive_failed_postprocessing_consumes_budget() -> Fallible<()> {
    let rr = make_randomized_response_bool(0.75, false)?;
    let budget = rr.map(&1)?;
    let failing = (rr.clone()
        >> Function::new_fallible(|_: &bool| {
            // A constructor called after a private release may fail.
            make_randomized_response_bool(-1.0, false)?;
            Ok(false)
        }))?;
    let mut qbl = make_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        1,
        vec![budget],
    )?
    .invoke(&true)?;

    assert!(qbl.eval(&failing).is_err());
    let error = qbl.eval(&rr).unwrap_err();
    assert_eq!(error.message.as_deref(), Some("out of queries"));
    Ok(())
}

#[test]
fn test_adaptive_pre_invocation_rejection_preserves_budget() -> Fallible<()> {
    let rr = make_randomized_response_bool(0.75, false)?;
    let mut qbl = make_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        1,
        vec![rr.map(&1)?],
    )?
    .invoke(&true)?;
    assert!(
        qbl.eval(&make_randomized_response_bool(0.9, false)?)
            .is_err()
    );
    assert!(qbl.eval(&rr).is_ok());
    Ok(())
}

#[test]
fn test_adaptive_failure_retires_previous_sequential_child() -> Fallible<()> {
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
    let mut qbl = make_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        1,
        vec![(1.0, 1e-7); 2],
    )?
    .invoke(&true)?;

    let mut previous: Queryable<(), bool> = qbl.eval_poly(&child.into_poly())?;
    assert!(previous.eval(&())?);
    assert!(qbl.eval(&failing.into_poly()).is_err());
    assert!(previous.eval(&()).is_err());
    Ok(())
}

#[test]
fn test_adaptive_failed_postprocessing_child_keeps_sequential_wrapper() -> Fallible<()> {
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
    let mut qbl = make_adaptive_composition::<_, _, _, Box<dyn Any>>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        1,
        vec![(1.0, 1e-7); 2],
    )?
    .invoke(&true)?;

    assert!(qbl.eval(&failing.into_poly()).is_err());
    let mut current: Queryable<(), bool> = qbl.eval_poly(&child.into_poly())?;
    assert!(current.eval(&())?);
    assert!(escaped.borrow_mut().as_mut().unwrap().eval(&()).is_err());
    Ok(())
}
