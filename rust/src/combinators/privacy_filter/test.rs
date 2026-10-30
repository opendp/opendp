use crate::core::Function;
use crate::{
    combinators::{CompositionMeasure, make_fully_adaptive_composition},
    domains::AtomDomain,
    error::Fallible,
    measurements::make_randomized_response_bool,
    measures::MaxDivergence,
    metrics::DiscreteDistance,
};

use super::make_privacy_filter;

#[test]
fn test_privacy_filter() -> Fallible<()> {
    let odom_comp = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        None,
    )?;

    let meas_filter = make_privacy_filter(odom_comp, 1, 1.0)?;
    let mut qbl_filter = meas_filter.invoke(&true)?;

    // ln(p / (1 - p)) = ln(0.51 / (1 - 0.51))
    let q1 = qbl_filter.invoke(make_randomized_response_bool(0.51, false)?);
    assert!(q1.is_ok());
    assert_eq!(qbl_filter.privacy_loss(1)?, 0.040005334613699206);

    // ...now add: ln(p / (1 - p)) = ln(0.51 / (1 - 0.51))
    let q2 = qbl_filter.invoke(make_randomized_response_bool(0.51, false)?);
    assert!(q2.is_ok());
    assert_eq!(qbl_filter.privacy_loss(1)?, 0.08001066922739841);

    // ...now try adding: ln(0.75 / (1 - 0.75))
    let q3 = qbl_filter.invoke(make_randomized_response_bool(0.75, false)?);
    assert!(q3.is_err());
    assert!(qbl_filter.privacy_loss(1).is_err());

    Ok(())
}

#[test]
fn test_privacy_filter_constructor_d_in() -> Fallible<()> {
    let odometer = |d_in| {
        make_fully_adaptive_composition(
            AtomDomain::<bool>::default(),
            DiscreteDistance,
            MaxDivergence,
            Some(d_in),
        )
    };
    let m_rr = make_randomized_response_bool(0.51, false)?;

    let mut qbl = make_privacy_filter(odometer(1)?, 1, 1.0)?.invoke(&true)?;
    assert!(qbl.invoke(m_rr.clone()).is_ok());

    let mut qbl = make_privacy_filter(odometer(2)?, 1, 1.0)?.invoke(&true)?;
    assert!(qbl.invoke(m_rr).is_err());
    Ok(())
}

#[test]
fn test_privacy_filter_failed_postprocessing_counts_privacy_loss() -> Fallible<()> {
    let rr = make_randomized_response_bool(0.75, false)?;
    let budget = rr.map(&1)?;
    let failing = (rr.clone()
        >> Function::new_fallible(|_: &bool| {
            make_randomized_response_bool(-1.0, false)?;
            Ok(false)
        }))?;
    let odometer = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        None,
    )?;
    let mut qbl = make_privacy_filter(odometer, 1, 3.0)?.invoke(&true)?;

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
fn test_privacy_filter_exhaustion_withholds_postprocessing_error() -> Fallible<()> {
    let failing = (make_randomized_response_bool(0.75, false)?
        >> Function::new_fallible(|_: &bool| {
            make_randomized_response_bool(-1.0, false)?;
            Ok(false)
        }))?;
    let odometer = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        None,
    )?;
    let mut qbl = make_privacy_filter(odometer, 1, 0.0)?.invoke(&true)?;

    // The error produced after the private release is also an answer that must
    // be withheld when the invocation exceeds the filter's budget.
    let error = qbl.invoke(failing.clone()).unwrap_err();
    assert!(error.message.unwrap().contains("filter is now exhausted"));
    let error = qbl.invoke(failing).unwrap_err();
    assert!(error.message.unwrap().contains("filter is exhausted"));
    Ok(())
}

#[test]
fn test_privacy_filter_failed_postprocessing_cannot_refund_budget() -> Fallible<()> {
    let rr = make_randomized_response_bool(0.75, false)?;
    let failing = (rr.clone()
        >> Function::new_fallible(|_: &bool| {
            make_randomized_response_bool(-1.0, false)?;
            Ok(false)
        }))?;
    let odometer = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
        None,
    )?;
    let mut qbl = make_privacy_filter(odometer, 1, rr.map(&1)?)?.invoke(&true)?;

    assert!(qbl.invoke(failing).is_err());
    assert!(qbl.invoke(rr).is_err());
    assert!(qbl.privacy_loss(1).is_err());
    Ok(())
}
