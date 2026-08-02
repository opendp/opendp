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
    for points in [vec![(f64::NAN, 0.1)], vec![(1.0, -0.1)], vec![(1.0, 1.1)]] {
        let error = PrivacyProfile::new(|_| Ok(1.0))
            .with_approxDP(points)
            .err()
            .unwrap();
        assert_eq!(error.variant, ErrorVariant::FailedMap);
    }
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
