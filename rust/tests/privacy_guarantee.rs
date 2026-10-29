use opendp::{
    error::Fallible,
    measures::{PrivacyGuarantee, PrivacyProfile},
};

// This external-crate test also runs without honest-but-curious enabled.
#[test]
fn public_builders_are_ungated() -> Fallible<()> {
    let profile = PrivacyProfile::new(|epsilon| Ok(vec![1.0; epsilon.len()]));
    let guarantee = PrivacyGuarantee::new()
        .with_approx_dp(1.0, 0.1)?
        .with_profile(profile);
    assert_eq!(guarantee.delta(1.0)?, 0.1);
    assert_eq!(guarantee.epsilon(0.1)?, 1.0);
    assert!(PrivacyGuarantee::new().with_approx_dp(-1.0, 0.0).is_err());
    for tradeoff in [
        PrivacyGuarantee::new().with_tradeoff(|alpha| Ok(vec![0.25; alpha.len()]))?,
        PrivacyGuarantee::new().with_symmetric_tradeoff(|alpha| Ok(vec![0.25; alpha.len()]))?,
    ] {
        assert_eq!(tradeoff.beta(&[0.0, 0.5])?, vec![0.25, 0.25]);
        assert_eq!(tradeoff.alpha(&[0.25])?, vec![0.0]);
        assert!(tradeoff.delta(0.0)? >= 0.75);
    }
    Ok(())
}
