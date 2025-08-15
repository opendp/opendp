use crate::{core::Function, domains::AtomDomain, metrics::DiscreteDistance, traits::InfExp};

use super::*;

#[test]
fn test_fix_delta_adp() -> Fallible<()> {
    let meas = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        SmoothedMaxDivergence,
        Function::new(|&v| v),
        PrivacyMap::new(|_d_in| PrivacyProfile::new(|eps| (-eps).inf_exp())),
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
        Approximate(SmoothedMaxDivergence),
        Function::new(|&v| v),
        PrivacyMap::new(|_d_in| (PrivacyProfile::new(|eps| (-eps).inf_exp()), 1e-7)),
    )?;
    let m_fixed = make_fix_delta(&meas, 2e-7)?;

    let (eps, del) = m_fixed.map(&1)?;

    // -ln(1e-7)
    assert_eq!(eps, 16.11809565095832);
    assert_eq!(del, 2e-7);
    Ok(())
}
