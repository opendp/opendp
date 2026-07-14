use std::any::Any;

use crate::{
    combinators::make_adaptive_composition,
    domains::AtomDomain,
    measurements::make_randomized_response_bool,
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
    metrics::DiscreteDistance,
    traits::InfAdd,
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

/// Under a Sequential measure, spawning a new query locks earlier interactive children.
fn assert_sequentiality_enforced(
    odometer: Odometer<
        AtomDomain<bool>,
        DiscreteDistance,
        Approximate<ZeroConcentratedDivergence>,
        Measurement<
            AtomDomain<bool>,
            DiscreteDistance,
            Approximate<ZeroConcentratedDivergence>,
            Queryable<(), bool>,
        >,
        Queryable<(), bool>,
    >,
) -> Fallible<()> {
    let m_interactive = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        Function::new_fallible(|&arg: &bool| Queryable::new_external(move |_: &()| Ok(!arg))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )?;

    let mut qbl = odometer.invoke(&false)?;
    let mut child = qbl.invoke(m_interactive.clone())?;
    // the child answers while it is the most recent query
    assert!(child.eval(&())?);
    // a new root query locks it
    let _child_2 = qbl.invoke(m_interactive)?;
    assert!(child.eval(&()).is_err());
    Ok(())
}

#[test]
fn test_sequentiality_enforced() -> Fallible<()> {
    assert_sequentiality_enforced(make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
    )?)
}

#[test]
fn test_sequentiality_enforced_k() -> Fallible<()> {
    assert_sequentiality_enforced(make_fully_adaptive_composition_k(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
    )?)
}

/// a renyi measurement whose curve counts its own evaluations
fn make_counting_renyi_measurement(
    eval_count: Rc<RefCell<usize>>,
    rho: f64,
) -> Fallible<Measurement<AtomDomain<bool>, DiscreteDistance, RenyiDivergence, bool>> {
    let curve = Function::new(move |alpha: &f64| {
        *eval_count.borrow_mut() += 1;
        rho * alpha
    });
    Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        RenyiDivergence,
        Function::new(|&arg: &bool| arg),
        PrivacyMap::new(move |_d_in: &u32| curve.clone()),
    )
}

#[test]
fn test_renyi_curve_evaluated_once_per_distinct_map() -> Fallible<()> {
    let count_1 = Rc::new(RefCell::new(0));
    let count_2 = Rc::new(RefCell::new(0));
    let m1 = make_counting_renyi_measurement(count_1.clone(), 0.5)?;
    let m2 = make_counting_renyi_measurement(count_2.clone(), 0.25)?;

    let odometer = make_fully_adaptive_composition_k::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        RenyiDivergence,
    )?;
    let mut qbl = odometer.invoke(&true)?;

    let k = 50;
    for _ in 0..k {
        qbl.invoke(m1.clone())?;
        qbl.invoke(m2.clone())?;
    }

    let curve = qbl.privacy_loss(1)?;
    assert_eq!(curve.eval(&2.0)?, (k as f64) * 1.0 + (k as f64) * 0.5);
    assert_eq!(*count_1.borrow(), 1);
    assert_eq!(*count_2.borrow(), 1);
    Ok(())
}

#[test]
fn test_uniform_run_matches_fully_adaptive_exactly() -> Fallible<()> {
    let m = make_counting_renyi_measurement(Rc::new(RefCell::new(0)), 0.5)?;

    let odometer = make_fully_adaptive_composition_k::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        RenyiDivergence,
    )?;
    let mut qbl = odometer.invoke(&true)?;

    let odometer_ref = make_fully_adaptive_composition::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        RenyiDivergence,
    )?;
    let mut qbl_ref = odometer_ref.invoke(&true)?;

    // one distinct map: identical fold order, so the loss is bit-identical
    for _ in 0..100 {
        qbl.invoke(m.clone())?;
        qbl_ref.invoke(m.clone())?;
    }
    assert_eq!(
        qbl.privacy_loss(1)?.eval(&2.0)?,
        qbl_ref.privacy_loss(1)?.eval(&2.0)?
    );
    Ok(())
}

#[test]
fn test_interleaved_queries_group_by_distinct_map() -> Fallible<()> {
    let m1 = make_randomized_response_bool(0.75, false)?;
    let m2 = make_randomized_response_bool(0.6, false)?;
    let sequence = [&m1, &m1, &m2, &m1];

    let odometer = make_fully_adaptive_composition_k::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
    )?;
    let mut qbl = odometer.invoke(&true)?;
    for m in sequence {
        qbl.invoke(m.clone())?;
    }

    // grouped fold: compose([compose_k(e1, 3), compose_k(e2, 1)])
    let e1 = m1.map(&1)?;
    let e2 = m2.map(&1)?;
    let expected = e1.inf_add(&e1)?.inf_add(&e1)?.inf_add(&e2)?;
    assert_eq!(qbl.privacy_loss(1)?, expected);

    // agrees with make_fully_adaptive_composition up to inf_add reordering
    let odometer_ref = make_fully_adaptive_composition::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
    )?;
    let mut qbl_ref = odometer_ref.invoke(&true)?;
    for m in sequence {
        qbl_ref.invoke(m.clone())?;
    }
    assert!((qbl.privacy_loss(1)? - qbl_ref.privacy_loss(1)?).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_privacy_loss_before_any_queries() -> Fallible<()> {
    let odometer = make_fully_adaptive_composition_k::<_, _, _, bool>(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
    )?;
    let mut qbl = odometer.invoke(&true)?;
    assert_eq!(qbl.privacy_loss(1)?, 0.0);
    Ok(())
}
