use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{MultiDP, PrivacyGuarantee, PrivacyProfile, Purity},
    metrics::AbsoluteDistance,
};

use super::make_multiDP_to_approxDP;

#[test]
fn test_cast_preserves_discontinuous_delta_endpoint() -> Fallible<()> {
    let profile = PrivacyProfile::new_log(|epsilons| {
        Ok(epsilons
            .iter()
            .map(|&epsilon| {
                if epsilon < 1.0 {
                    0.0
                } else {
                    f64::NEG_INFINITY
                }
            })
            .collect())
    })
    .with_delta_slack(0.1)?;
    let guarantee = PrivacyGuarantee::new().with_profile(profile);
    let measurement = Measurement::new(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        MultiDP::with_profile(Purity::Approximate),
        Function::new(|x: &f64| *x),
        PrivacyMap::new_fallible(move |_| Ok(guarantee.clone())),
    )?;

    let cast = make_multiDP_to_approxDP(measurement.clone(), 0.1)?;
    assert_eq!(cast.map(&1.0)?, (1.0, 0.1));
    let cast = make_multiDP_to_approxDP(measurement.clone(), 0.1_f64.next_down())?;
    assert!(cast.map(&1.0)?.0.is_infinite());
    for delta in [-0.1, 1.1, f64::NAN] {
        assert!(make_multiDP_to_approxDP(measurement.clone(), delta).is_err());
    }
    Ok(())
}
