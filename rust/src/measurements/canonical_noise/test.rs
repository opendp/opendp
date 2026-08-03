use crate::{
    domains::AtomDomain, error::Fallible, measures::PrivacyGuarantee, metrics::AbsoluteDistance,
};

use super::make_canonical_noise;

fn symmetric_tradeoff() -> Fallible<PrivacyGuarantee> {
    PrivacyGuarantee::new()
        .with_symmetric_tradeoff(|alpha| Ok((1.0 - 2.0 * alpha).max((1.0 - alpha) / 2.0).max(0.0)))
}

#[test]
fn test_canonical_noise() -> Fallible<()> {
    let m_cnd = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        1.,
        symmetric_tradeoff()?,
    )?;
    assert!(m_cnd.invoke(&1.).is_ok());
    assert!(m_cnd.map(&1.)?.beta(0.25)? > 0.0);
    Ok(())
}

#[test]
fn test_canonical_noise_uses_only_symmetric_tradeoff_view() -> Fallible<()> {
    let d_out = symmetric_tradeoff()?.with_zCDP(0.1, 0.2)?;
    let m_cnd = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        2.0,
        d_out,
    )?;

    let partial = m_cnd.map(&1.0)?;
    assert_eq!(partial.beta(0.25)?, 0.5);
    let debug = format!("{partial:?}");
    assert!(debug.contains("tradeoff: true"));
    assert!(debug.contains("profile: false"));
    assert!(debug.contains("renyi: false"));
    assert!(debug.contains("zcdp: false"));
    Ok(())
}

#[test]
fn test_canonical_noise_zero_distance_is_perfect_privacy() -> Fallible<()> {
    let m_cnd = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        1.0,
        symmetric_tradeoff()?,
    )?;
    let zero = m_cnd.map(&0.0)?;
    assert_eq!(zero.beta(0.25)?, 0.75);
    assert_eq!(zero.beta(0.75)?, 0.25);
    Ok(())
}

#[test]
fn test_canonical_noise_rejects_nonsymmetric_tradeoff() -> Fallible<()> {
    let d_out = PrivacyGuarantee::new().with_tradeoff(|alpha| Ok((0.8 - alpha).max(0.0)))?;
    assert!(
        make_canonical_noise(
            AtomDomain::new_non_nan(),
            AbsoluteDistance::default(),
            1.0,
            d_out,
        )
        .is_err()
    );
    Ok(())
}
