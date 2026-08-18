use super::{MultiDP, PrivacyGuarantee, PrivacyProfile, Purity};
use crate::error::Fallible;

#[test]
fn purity_ordering_is_approximate_before_pure() {
    assert!(Purity::Approximate < Purity::Pure);
    assert!(None::<Purity> < Some(Purity::Approximate));
    assert!(Some(Purity::Approximate) < Some(Purity::Pure));
}

#[test]
fn multidp_equality_ignores_capabilities() {
    let pure = MultiDP::with_profile(Purity::Pure);
    let approximate = MultiDP::with_profile(Purity::Approximate);
    let empty = MultiDP::default();

    let renyi_approximate = MultiDP::with_renyi(Purity::Approximate);
    let renyi_pure = MultiDP::with_renyi(Purity::Pure);

    assert_eq!(pure, approximate);
    assert_eq!(pure, empty);
    assert_eq!(renyi_approximate, renyi_pure);
    assert_ne!(pure.capabilities(), approximate.capabilities());
    assert_ne!(pure.capabilities(), empty.capabilities());
}

#[test]
fn multidp_capabilities_are_inspectable() {
    let pure = MultiDP::with_profile(Purity::Pure);
    let approximate = MultiDP::with_profile(Purity::Approximate);
    let empty = MultiDP::default();
    let tradeoff = MultiDP::with_tradeoff();
    let renyi_approximate = MultiDP::with_renyi(Purity::Approximate);
    let renyi_pure = MultiDP::with_renyi(Purity::Pure);
    assert_eq!(pure.capabilities().profile(), Some(Purity::Pure));
    assert_eq!(
        approximate.capabilities().profile(),
        Some(Purity::Approximate)
    );
    assert_eq!(empty.capabilities().profile(), None);
    assert!(!empty.capabilities().tradeoff());
    assert!(tradeoff.capabilities().tradeoff());
    assert_eq!(empty.capabilities().renyi(), None);
    assert_eq!(
        renyi_approximate.capabilities().renyi(),
        Some(Purity::Approximate)
    );
    assert_eq!(renyi_pure.capabilities().renyi(), Some(Purity::Pure));
    assert!(format!("{pure:?}").contains("Pure"));
    assert!(format!("{approximate:?}").contains("Approximate"));
}

#[test]
fn profile_only_guarantee_delegates_queries() -> Fallible<()> {
    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(1.0, 0.1), (2.0, 0.0)])?;
    let guarantee = PrivacyGuarantee::new().with_profile(profile);

    assert_eq!(guarantee.delta(1.0)?, 0.1);
    assert_eq!(guarantee.epsilon(0.1)?, 1.0);
    assert!(guarantee.profile().is_some());
    Ok(())
}

#[test]
fn empty_guarantee_has_no_profile() {
    let guarantee = PrivacyGuarantee::new();
    assert!(guarantee.delta(0.0).is_err());
    assert!(guarantee.epsilon(0.5).is_err());
}
