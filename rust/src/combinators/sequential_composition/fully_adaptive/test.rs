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
    )?;

    let mut qbl = m_odo.invoke(&false)?;
    assert!(qbl.invoke(m_query)?);
    assert_eq!(qbl.privacy_loss(1)?, (1.0, 1e-7));
    Ok(())
}
