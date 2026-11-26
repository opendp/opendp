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
fn test_multidp_fixed_budgets_check_before_release() -> Fallible<()> {
    use crate::measures::{MultiDP, PrivacyBudget, PrivacyGuarantee};

    let compositor = make_adaptive_composition::<_, _, _, bool>(
        AtomDomain::<bool>::default(), DiscreteDistance, MultiDP::default(), 1,
        vec![PrivacyBudget::ApproxDP(0.5, 0.0), PrivacyBudget::ZCDP(0.1, 0.0)],
    )?;
    let child = |epsilon: f64, rho: f64| {
        Measurement::new(
            AtomDomain::<bool>::default(), DiscreteDistance, MultiDP::default(),
            Function::new(|_: &bool| false),
            PrivacyMap::new_fallible(move |_| PrivacyGuarantee::new()
                .with_approx_dp(epsilon, 0.0)?.with_zCDP(rho, 0.0)),
        )
    };
    let mut queryable = compositor.invoke(&false)?;
    assert!(queryable.eval(&child(1.0, 0.2)?).is_err());
    assert!(!queryable.eval(&child(0.5, 0.2)?)?);
    assert!(queryable.eval(&child(0.0, 0.2)?).is_err());
    assert!(!queryable.eval(&child(1.0, 0.1)?)?);
    assert!(queryable.eval(&child(0.0, 0.0)?).is_err());
    assert!(compositor.map(&1)?.delta(1.0)? < 1.0);
    Ok(())
}
