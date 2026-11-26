use super::{AccountingPreference, MultiDP, PrivacyGuarantee, PrivacyProfile};
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
fn multidp_equality_ignores_preferences() {
    let auto = MultiDP::default();
    for preference in [AccountingPreference::PureDp, AccountingPreference::Zcdp] {
        let measure = MultiDP::new(preference);
        assert_eq!(measure, auto);
        assert_eq!(measure.preference(), preference);
    }
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
                assert!(guarantee.delta(epsilon.next_down_())? >= delta);
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
fn approximate_dp_symmetric_geometry() -> Fallible<()> {
    let epsilon0 = 2.0_f64;
    let delta0 = 0.1_f64;
    let guarantee = PrivacyGuarantee::new().with_approx_dp(epsilon0, delta0)?;
    let epsilons = [0.0_f64, 0.5, 1.0, 2.0];
    let bounds = guarantee.delta_batch(&epsilons)?;
    let logs = guarantee.log_delta_batch(&epsilons)?;
    for ((epsilon, delta), log_delta) in epsilons.into_iter().zip(bounds).zip(logs) {
        let expected = 1.0 - (1.0 - delta0) * (1.0 + epsilon.exp()) / (1.0 + epsilon0.exp());
        assert!((delta - expected).abs() < 1e-14);
        assert!((log_delta.exp() - delta).abs() < 1e-14);
        let inverse = guarantee.epsilon(delta)?;
        assert!((inverse - epsilon).abs() < 1e-13);
        assert!(guarantee.delta(inverse)? <= delta + 1e-14);
        let inverse_log = guarantee.epsilon_from_log_delta(log_delta)?;
        assert!((inverse_log - epsilon).abs() < 1e-13);
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
    assert_eq!(empty.delta_batch(&[0.0, f64::INFINITY])?, vec![1.0, 1.0]);
    assert_eq!(empty.log_delta(0.0)?, 0.0);
    assert_eq!(empty.epsilon_batch(&[0.0, 1.0])?, vec![f64::INFINITY, 0.0]);
    assert_eq!(empty.epsilon_from_log_delta(-1.0)?, f64::INFINITY);
    assert_eq!(empty.epsilon_from_log_delta(0.0)?, 0.0);
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
