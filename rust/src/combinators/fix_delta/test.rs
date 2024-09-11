use crate::{core::Function, domains::AtomDomain, metrics::DiscreteDistance};

use super::*;

#[test]
fn test_fix_delta_adp() -> Fallible<()> {
    let meas = Measurement::new(
        AtomDomain::<bool>::default(),
        Function::new(|&v| v),
        DiscreteDistance,
        SmoothedMaxDivergence,
        PrivacyMap::new(|_d_in| PrivacyProfile::new(|eps| Ok(eps))),
    )?;
    let m_fixed = make_fix_delta(&meas, 1e-7)?;

    let (eps, del) = m_fixed.map(&1)?;

    assert_eq!(eps, 1e-7);
    assert_eq!(del, 1e-7);
    Ok(())
}

#[test]
fn test_fix_delta_approx_adp() -> Fallible<()> {
    let meas = Measurement::new(
        AtomDomain::<bool>::default(),
        Function::new(|&v| v),
        DiscreteDistance,
        Approximate(SmoothedMaxDivergence),
        PrivacyMap::new(|_d_in| (PrivacyProfile::new(|eps| Ok(eps)), 1e-7)),
    )?;
    let m_fixed = make_fix_delta(&meas, 2e-7)?;

    let (eps, del) = m_fixed.map(&1)?;

    assert_eq!(eps, 1e-7);
    assert_eq!(del, 2e-7);
    Ok(())
}
