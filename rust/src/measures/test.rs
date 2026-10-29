use super::{MultiDP, PrivacyCapabilities, PrivacyGuarantee, PrivacyProfile, Purity};
use crate::{error::Fallible, traits::NextFloat};

fn staircase(epsilon: f64, delta: f64) -> Fallible<PrivacyProfile> {
    PrivacyProfile::new(move |values| {
        Ok(values
            .iter()
            .map(|&value| if value < epsilon { 1.0 } else { 0.0 })
            .collect())
    })
    .with_delta_slack(delta)
}

#[test]
fn purity_ordering_is_approximate_before_pure() {
    assert!(Purity::Approximate < Purity::Pure);
    assert!(None::<Purity> < Some(Purity::Approximate));
    assert!(Some(Purity::Approximate) < Some(Purity::Pure));
}

#[test]
fn multidp_equality_ignores_capabilities() {
    let empty = MultiDP::default();
    for measure in [
        MultiDP::with_approx_dp(Purity::Pure),
        MultiDP::with_approx_dp(Purity::Approximate),
        MultiDP::with_profile(Purity::Pure),
        MultiDP::with_profile(Purity::Approximate),
        MultiDP::new(PrivacyCapabilities::new(
            Some(Purity::Pure),
            Some(Purity::Approximate),
        )),
    ] {
        assert_eq!(measure, empty);
        assert_ne!(measure.capabilities(), empty.capabilities());
    }
}

#[test]
fn multidp_capabilities_are_independent_presence_promises() -> Fallible<()> {
    let empty = MultiDP::default();
    assert_eq!(empty.capabilities().approx_dp(), None);
    assert_eq!(empty.capabilities().profile(), None);
    for purity in [Purity::Pure, Purity::Approximate] {
        let certificate_only = MultiDP::with_approx_dp(purity);
        assert_eq!(certificate_only.capabilities().approx_dp(), Some(purity));
        assert_eq!(certificate_only.capabilities().profile(), None);
        let profile_only = MultiDP::with_profile(purity);
        assert_eq!(profile_only.capabilities().approx_dp(), None);
        assert_eq!(profile_only.capabilities().profile(), Some(purity));
    }
    let both = MultiDP::new(PrivacyCapabilities::new(
        Some(Purity::Pure),
        Some(Purity::Approximate),
    ));
    assert_eq!(both.capabilities().approx_dp(), Some(Purity::Pure));
    assert_eq!(both.capabilities().profile(), Some(Purity::Approximate));
    assert!(format!("{both:?}").contains("Pure"));

    // An evaluation may return more representations than its measure promises.
    let measure = MultiDP::with_approx_dp(Purity::Pure);
    let guarantee = PrivacyGuarantee::new()
        .with_approx_dp(1.0, 0.0)?
        .with_profile(staircase(1.0, 0.0)?);
    assert!(guarantee.profile().is_some());
    assert_eq!(measure.capabilities().profile(), None);
    Ok(())
}

#[test]
fn approximate_dp_preserves_exact_certificates_without_materializing_profiles() -> Fallible<()> {
    for epsilon in [0.0, f64::from_bits(1), 1.0, f64::MAX] {
        for delta in [0.0, f64::from_bits(1), 0.1, 0.5, 1.0] {
            let guarantee = PrivacyGuarantee::new().with_approx_dp(epsilon, delta)?;
            assert_eq!(guarantee.approx_dp(), Some((epsilon, delta)));
            assert_eq!(guarantee.delta(epsilon)?, delta);
            assert_eq!(guarantee.delta(f64::INFINITY)?, delta);
            assert_eq!(
                guarantee.epsilon(delta)?,
                if delta == 1.0 { 0.0 } else { epsilon }
            );
            assert_eq!(guarantee.epsilon(1.0)?, 0.0);
            if epsilon > 0.0 {
                assert_eq!(guarantee.delta(epsilon.next_down_())?, 1.0);
            }
            if delta > 0.0 {
                assert_eq!(guarantee.epsilon(delta.next_down_())?, f64::INFINITY);
            }
            assert!(guarantee.profile().is_none());
        }
    }
    Ok(())
}

#[test]
fn attaching_each_representation_preserves_the_other() -> Fallible<()> {
    let profile = staircase(2.0, 0.0)?;
    for guarantee in [
        PrivacyGuarantee::new()
            .with_approx_dp(1.0, 0.1)?
            .with_profile(profile.clone()),
        PrivacyGuarantee::new()
            .with_profile(profile)
            .with_approx_dp(1.0, 0.1)?,
    ] {
        assert_eq!(guarantee.approx_dp(), Some((1.0, 0.1)));
        assert!(guarantee.profile().is_some());
        // The certificate is tighter at epsilon=1, the profile at epsilon=2.
        assert_eq!(guarantee.delta(1.0)?, 0.1);
        assert_eq!(guarantee.delta(2.0)?, 0.0);
        assert_eq!(guarantee.epsilon(0.1)?, 1.0);
        assert_eq!(guarantee.epsilon(0.0)?, 2.0);
        // Attaching the certificate did not give the profile its source slack.
        assert_eq!(guarantee.profile().unwrap().delta(&[2.0])?[0], 0.0);
    }
    Ok(())
}

#[test]
fn profile_only_guarantee_delegates_queries() -> Fallible<()> {
    let profile = staircase(1.0, 0.1)?;
    let guarantee = PrivacyGuarantee::new().with_profile(profile);
    assert_eq!(guarantee.delta(1.0)?, 0.1);
    assert_eq!(guarantee.epsilon(0.1)?, 1.0);
    assert!(guarantee.profile().is_some());
    assert!(guarantee.approx_dp().is_none());
    Ok(())
}

#[test]
fn missing_and_failed_representations() -> Fallible<()> {
    let empty = PrivacyGuarantee::new();
    assert!(empty.approx_dp().is_none());
    assert!(empty.profile().is_none());
    assert!(empty.delta(0.0).is_err());
    assert!(empty.epsilon(1.0).is_err());
    let failed = PrivacyGuarantee::new().with_profile(PrivacyProfile::new_log(|_| {
        fallible!(FailedFunction, "profile failure")
    }));
    for guarantee in [failed.clone(), failed.with_approx_dp(1.0, 0.1)?] {
        assert_eq!(
            guarantee.delta(1.0).unwrap_err().message.as_deref(),
            Some("profile failure")
        );
        assert_eq!(
            guarantee.epsilon(0.1).unwrap_err().message.as_deref(),
            Some("profile failure")
        );
    }
    Ok(())
}

#[test]
fn certificates_and_queries_validate_ranges() -> Fallible<()> {
    for invalid in [-1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        assert!(
            PrivacyGuarantee::new()
                .with_approx_dp(invalid, 0.0)
                .is_err()
        );
    }
    let guarantee = PrivacyGuarantee::new().with_approx_dp(1.0, 0.0)?;
    for invalid in [-1.0, f64::NEG_INFINITY, f64::NAN] {
        assert!(guarantee.delta(invalid).is_err());
    }
    for invalid in [-1.0, 1.1, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        assert!(
            PrivacyGuarantee::new()
                .with_approx_dp(1.0, invalid)
                .is_err()
        );
        assert!(guarantee.epsilon(invalid).is_err());
    }
    Ok(())
}
