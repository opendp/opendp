use crate::{
    domains::AtomDomain, error::Fallible, measures::PrivacyCurve, metrics::AbsoluteDistance,
};

use super::make_canonical_noise;

#[test]
fn test_canonical_noise_approxdp() -> Fallible<()> {
    let d_out = PrivacyCurve::new_approxdp(vec![(0.1, 1e-7)])?;

    let m_cnd = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        1.,
        d_out.clone(),
    )?;
    assert!(m_cnd.invoke(&1.).is_ok());
    assert_eq!(m_cnd.map(&1.)?.beta(0.1)?, d_out.beta(0.1)?);
    Ok(())
}

#[test]
fn test_canonical_noise_fdp_exact_d_in() -> Fallible<()> {
    let d_out = PrivacyCurve::new_approxdp(vec![(0.1, 1e-7)])?;

    let m_cnd = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        1.,
        d_out.clone(),
    )?;

    let mapped_full = m_cnd.map(&1.0)?;
    for alpha in [0.1, 0.3, 0.7] {
        let expected = d_out.beta(alpha)?;
        let full = mapped_full.beta(alpha)?;

        assert!((full - expected).abs() < 1e-12);
    }

    assert!(m_cnd.map(&0.0).is_err());
    assert!(m_cnd.map(&0.5).is_err());

    Ok(())
}
