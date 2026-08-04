use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;
use crate::error::ErrorVariant;

#[test]
fn test_points_and_function_queries_agree() -> Fallible<()> {
    let points = PrivacyProfile::new(|epsilon| {
        Ok(if epsilon < 1.0 {
            1.0
        } else if epsilon < 3.0 {
            0.2
        } else {
            0.01
        })
    })
    .with_approxDP(vec![(3.0, 0.01), (1.0, 0.2)])?;
    let function = PrivacyProfile::new(|epsilon| {
        Ok(if epsilon < 1.0 {
            1.0
        } else if epsilon < 3.0 {
            0.2
        } else {
            0.01
        })
    });

    for epsilon in [0.0, 0.5, 1.0, 2.0, 3.0, 10.0] {
        assert!((points.delta(epsilon)? - function.delta(epsilon)?).abs() < 1e-15);
    }
    for delta in [0.5, 0.1, 0.001] {
        assert_eq!(points.epsilon(delta)?, function.epsilon(delta)?);
    }
    Ok(())
}

#[test]
fn test_points_normalize_redundant_and_plateau_points() -> Fallible<()> {
    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![
        (3.0, 0.2), // dominated by the point at epsilon 1
        (1.0, 0.5),
        (1.0, 0.2), // duplicate epsilon: retain the tightest bound
        (0.0, 0.8),
        (2.0, 0.3), // dominated by the point at epsilon 1
        (4.0, 0.0),
    ])?;

    assert_eq!(profile.delta(0.0)?, 0.8);
    assert_eq!(profile.delta(1.0)?, 0.2);
    assert_eq!(profile.delta(3.0)?, 0.2);
    assert_eq!(profile.delta(4.0)?, 0.0);
    assert_eq!(profile.epsilon(0.2)?, 1.0);
    assert_eq!(profile.epsilon(0.0)?, 4.0);
    Ok(())
}

#[test]
fn test_points_validate_individual_values() {
    let signed_zero = PrivacyProfile::new(|_| Ok(1.0))
        .with_approxDP(vec![(-0.0, -0.0)])
        .unwrap();
    assert_eq!(signed_zero.delta(-0.0).unwrap(), -0.0);

    for points in [vec![(f64::NAN, 0.1)], vec![(1.0, -0.1)], vec![(1.0, 1.1)]] {
        let error = PrivacyProfile::new(|_| Ok(1.0))
            .with_approxDP(points)
            .err()
            .unwrap();
        assert_eq!(error.variant, ErrorVariant::FailedMap);
    }
}

#[test]
fn test_function_inversion_handles_step_profiles() -> Fallible<()> {
    let profile = PrivacyProfile::new(|epsilon| Ok(if epsilon < 0.5 { 1.0 } else { 1e-8 }));
    let epsilon = profile.epsilon(1e-8)?;
    assert!(epsilon.is_finite());
    assert!((epsilon - 0.5).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_function_inversion_is_lazy() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let profile_calls = calls.clone();
    let profile = PrivacyProfile::new(move |epsilon| {
        profile_calls.fetch_add(1, Ordering::Relaxed);
        Ok((-epsilon).exp())
    });

    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let epsilon = profile.epsilon((-0.25f64).exp())?;
    assert!(epsilon >= 0.25);
    assert!((epsilon - 0.25).abs() < 1e-12);
    assert!(calls.load(Ordering::Relaxed) > 0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_independent_inverse_is_used_without_eager_evaluation() -> Fallible<()> {
    let forward_calls = Arc::new(AtomicUsize::new(0));
    let inverse_calls = Arc::new(AtomicUsize::new(0));
    let forward_counter = forward_calls.clone();
    let inverse_counter = inverse_calls.clone();

    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_log_profile_with_epsilon(
        move |epsilon| {
            forward_counter.fetch_add(1, Ordering::Relaxed);
            Ok(-epsilon)
        },
        move |delta| {
            inverse_counter.fetch_add(1, Ordering::Relaxed);
            Ok(-delta.ln())
        },
    )?;

    assert_eq!(forward_calls.load(Ordering::Relaxed), 0);
    assert_eq!(inverse_calls.load(Ordering::Relaxed), 0);
    assert_eq!(profile.epsilon(0.25)?, -0.25f64.ln());
    assert_eq!(inverse_calls.load(Ordering::Relaxed), 1);
    assert_eq!(forward_calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[test]
fn test_pure_epsilon_for_points_and_functions() -> Fallible<()> {
    let pure_points = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(1.5, 0.0)])?;
    assert_eq!(pure_points.pure_epsilon()?, Some(1.5));

    let non_pure_points = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(1.5, 1e-6)])?;
    assert_eq!(non_pure_points.pure_epsilon()?, None);

    let pure_function =
        PrivacyProfile::new(|epsilon| Ok(if epsilon < 2.0 { (-epsilon).exp() } else { 0.0 }));
    assert_eq!(pure_function.pure_epsilon()?, Some(2.0));
    Ok(())
}

#[test]
fn test_conservative_endpoint_behavior() -> Fallible<()> {
    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(1.0, 0.1)])?;
    assert_eq!(profile.epsilon(0.1)?, 1.0);
    assert_eq!(profile.epsilon(0.1f64.next_down())?, f64::INFINITY);
    assert!(profile.delta(0.9999999999999999)? >= 1.0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_pure_dp_profile_tradeoff_oracle() -> Fallible<()> {
    let epsilon_0 = 2.0f64.ln();
    let profile =
        PrivacyProfile::new(move |epsilon| Ok((1.0 - (epsilon - epsilon_0).exp()).max(0.0)));
    let guarantee = PrivacyGuarantee::from_profile(profile.clone());

    assert!((guarantee.beta(0.25)? - 0.5).abs() < 1e-10);
    assert!((guarantee.beta(0.75)? - 0.125).abs() < 1e-10);
    assert!(profile.delta(epsilon_0)? <= 1e-12);

    let tradeoff = PrivacyGuarantee::new().with_symmetric_tradeoff(|alpha| {
        Ok((1.0 - 2.0 * alpha).max((1.0 - alpha) / 2.0).max(0.0))
    })?;
    assert!(tradeoff.delta(epsilon_0)? <= 1e-10);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_aggregate_queries_are_monotone() -> Fallible<()> {
    let epsilon_0 = 2.0f64.ln();
    let profile =
        PrivacyProfile::new(move |epsilon| Ok((1.0 - (epsilon - epsilon_0).exp()).max(0.0)));
    let profile_only = PrivacyGuarantee::from_profile(profile.clone());
    let tradeoff_only =
        PrivacyGuarantee::new().with_symmetric_tradeoff(|alpha| Ok((0.85 - alpha).max(0.0)))?;
    let combined = PrivacyGuarantee::from_profile(profile)
        .with_symmetric_tradeoff(|alpha| Ok((0.85 - alpha).max(0.0)))?;

    let combined_delta = combined.delta(0.25)?;
    assert!(combined_delta <= profile_only.delta(0.25)? + 1e-10);
    assert!(combined_delta <= tradeoff_only.delta(0.25)? + 1e-10);

    let combined_epsilon = combined.epsilon(0.25)?;
    assert!(combined_epsilon <= profile_only.epsilon(0.25)? + 1e-10);
    assert!(combined_epsilon <= tradeoff_only.epsilon(0.25)? + 1e-10);

    let combined_beta = combined.beta(0.25)?;
    assert!(combined_beta + 1e-10 >= profile_only.beta(0.25)?);
    assert!(combined_beta + 1e-10 >= tradeoff_only.beta(0.25)?);

    let combined_alpha = combined.alpha(0.25)?;
    assert!(combined_alpha + 1e-10 >= profile_only.alpha(0.25)?);
    assert!(combined_alpha + 1e-10 >= tradeoff_only.alpha(0.25)?);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_tradeoff_and_profile_paths_are_both_available() -> Fallible<()> {
    let tradeoff =
        PrivacyGuarantee::new().with_symmetric_tradeoff(|alpha| Ok((0.75 - alpha).max(0.0)))?;
    assert_eq!(tradeoff.beta(0.25)?, 0.5);
    assert_eq!(tradeoff.alpha(0.25)?, 0.5);
    assert!(tradeoff.delta(1.0)? <= 1.0);

    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(1.0, 0.1), (2.0, 0.01)])?;
    let profile_only = PrivacyGuarantee::from_profile(profile.clone());
    assert!(profile_only.beta(0.5)? >= 0.0);

    let function_profile = PrivacyProfile::new(|epsilon| Ok((-epsilon).exp()));
    let function_guarantee = PrivacyGuarantee::from_profile(function_profile);
    assert!(function_guarantee.beta(0.5)? >= 0.0);

    let combined = PrivacyGuarantee::from_profile(profile).with_tradeoff(|_| Ok(0.9))?;
    assert!(combined.beta(0.5)? >= 0.9);
    assert!(combined.delta(1.0)? <= 0.1);

    let profile_error = PrivacyProfile::new(|_| fallible!(FailedFunction, "profile path failed"));
    let profile_error = PrivacyGuarantee::from_profile(profile_error).with_tradeoff(|_| Ok(0.5))?;
    assert_eq!(profile_error.beta(0.5)?, 0.5);

    let tradeoff_error = PrivacyGuarantee::from_profile(PrivacyProfile::new(|_| Ok(0.25)))
        .with_tradeoff(|_| fallible!(FailedFunction, "tradeoff path failed"))?;
    assert!(tradeoff_error.beta(0.5)? >= 0.0);

    let tradeoff_error = PrivacyGuarantee::new()
        .with_symmetric_tradeoff(|_| fallible!(FailedFunction, "delta path failed"))?;
    let error = tradeoff_error.delta(1.0).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::FailedFunction);
    Ok(())
}
