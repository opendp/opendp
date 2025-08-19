use crate::{
    combinators::make_fully_adaptive_composition, domains::AtomDomain, error::Fallible,
    measurements::make_randomized_response_bool, measures::MaxDivergence,
    metrics::DiscreteDistance,
};

use super::make_privacy_filter;

#[test]
fn test_privacy_filter() -> Fallible<()> {
    let odom_comp = make_fully_adaptive_composition(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MaxDivergence,
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
