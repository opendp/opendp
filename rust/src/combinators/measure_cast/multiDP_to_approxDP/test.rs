use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{MultiDP, PrivacyGuarantee, PrivacyProfile},
    metrics::AbsoluteDistance,
};

use super::make_multiDP_to_approxDP;

#[test]
fn test_cast_preserves_discontinuous_delta_endpoint() -> Fallible<()> {
    let guarantee = PrivacyGuarantee::from_profile(
        PrivacyProfile::new(|_| Ok(1.0))
            .with_approxDP(vec![(1.0, 0.1), (0.5, 0.1f64.next_up())])?,
    );
    let measurement = Measurement::new(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        MultiDP::default(),
        Function::new(|x: &f64| *x),
        PrivacyMap::new_fallible(move |_| Ok(guarantee.clone())),
    )?;

    let cast = make_multiDP_to_approxDP(measurement, 0.1)?;
    assert_eq!(cast.map(&1.0)?, (1.0, 0.1));
    Ok(())
}
