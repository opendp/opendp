use crate::{core::Function, domains::AtomDomain, measures::MultiDP, metrics::DiscreteDistance};

use super::*;

#[test]
fn test_fix_delta_adp() -> Fallible<()> {
    let meas = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        MultiDP,
        Function::new(|&v| v),
        PrivacyMap::new(|_d_in| {
            PrivacyGuarantee::new()
                .with_log_profile(|eps| Ok(-eps))
                .unwrap()
        }),
    )?;
    let m_fixed = make_fix_delta(&meas, 1e-7)?;

    let (eps, del) = m_fixed.map(&1)?;

    // -ln(1e-7)
    assert_eq!(eps, 16.11809565095832);
    assert_eq!(del, 1e-7);
    Ok(())
}

#[test]
fn test_fix_delta_approx_adp() -> Fallible<()> {
    let meas = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(MultiDP),
        Function::new(|&v| v),
        PrivacyMap::new(|_d_in| {
            (
                PrivacyGuarantee::new()
                    .with_log_profile(|eps| Ok(-eps))
                    .unwrap(),
                1e-7,
            )
        }),
    )?;
    let m_fixed = make_fix_delta(&meas, 2e-7)?;

    let (eps, del) = m_fixed.map(&1)?;

    // -ln(1e-7)
    assert_eq!(eps, 16.11809565095832);
    assert_eq!(del, 2e-7);
    assert!(make_fix_delta(&meas, -0.0)?.privacy_map.eval(&1).is_err());
    Ok(())
}

#[test]
fn test_fix_delta_outer_delta_boundaries() -> Fallible<()> {
    let meas = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(MultiDP),
        Function::new(|&v| v),
        PrivacyMap::new_fallible(|_d_in| {
            Ok((
                PrivacyGuarantee::new().with_approxDP(vec![(0.0, 0.0)])?,
                0.2,
            ))
        }),
    )?;
    let below = make_fix_delta(&meas, 0.2f64.next_down())?;
    assert!(below.privacy_map.eval(&1)?.0.is_infinite());

    let equal = make_fix_delta(&meas, 0.2)?;
    assert_eq!(equal.privacy_map.eval(&1)?.0, 0.0);

    let above = make_fix_delta(&meas, 0.2f64.next_up())?;
    assert_eq!(above.privacy_map.eval(&1)?.0, 0.0);
    Ok(())
}

#[test]
fn test_fix_delta_preserves_internal_rdp_delta() -> Fallible<()> {
    let meas = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(MultiDP),
        Function::new(|&v| v),
        PrivacyMap::new_fallible(|_d_in| {
            Ok((
                PrivacyGuarantee::new().with_renyiDP_trusted(|_| Ok(0.0), 0.1)?,
                0.2,
            ))
        }),
    )?;
    let equal = make_fix_delta(&meas, 0.2)?;
    assert!(equal.privacy_map.eval(&1)?.0.is_infinite());

    let above = make_fix_delta(&meas, 0.3f64.next_up())?;
    assert_eq!(above.privacy_map.eval(&1)?.0, 0.0);
    Ok(())
}
