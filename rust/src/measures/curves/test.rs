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

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_approxdp_points_have_certified_symmetric_tradeoff() -> Fallible<()> {
    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(1.0, 0.1), (2.0, 0.01)])?;
    let guarantee = PrivacyGuarantee::from_profile(profile).with_approxDP_tradeoff_trusted()?;
    let alpha = 0.2;
    let expected = [(1.0_f64, 0.1_f64), (2.0_f64, 0.01_f64)]
        .into_iter()
        .map(|(epsilon, delta)| {
            (0.0_f64)
                .max(1.0 - delta - epsilon.exp() * alpha)
                .max((-epsilon).exp() * (1.0 - delta - alpha))
        })
        .fold(0.0_f64, f64::max);

    let beta = guarantee.beta(alpha)?;
    assert!(beta <= expected);
    assert!(beta > 0.0);
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
fn test_renyi_representation_queries_and_aggregation() -> Fallible<()> {
    let exact = PrivacyGuarantee::new().with_renyiDP(|_| Ok(0.0), 0.0)?;
    let approximate = PrivacyGuarantee::new().with_renyiDP(|_| Ok(0.0), 0.1)?;

    assert_eq!(exact.delta(f64::INFINITY)?, 0.0);
    assert_eq!(approximate.delta(f64::INFINITY)?, 0.1);
    assert_eq!(exact.beta(0.5)?, 0.5);
    assert!((exact.alpha(0.5)? - 0.5).abs() < 1e-14);

    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(1.0, 0.2)])?;
    let profile_only = PrivacyGuarantee::from_profile(profile.clone());
    let rdp_only = PrivacyGuarantee::new().with_renyiDP(|_| Ok(0.0), 0.0)?;
    let combined =
        PrivacyGuarantee::from_profile(profile.clone()).with_renyiDP(|_| Ok(0.0), 0.0)?;

    assert!(combined.delta(1.0)? <= profile_only.delta(1.0)?);
    assert!(combined.delta(1.0)? <= rdp_only.delta(1.0)?);
    assert!(combined.epsilon(0.2)? <= profile_only.epsilon(0.2)?);
    assert!(combined.epsilon(0.2)? <= rdp_only.epsilon(0.2)?);
    assert!(combined.beta(0.5)? >= profile_only.beta(0.5)?);
    assert!(combined.beta(0.5)? >= rdp_only.beta(0.5)?);
    assert!(combined.alpha(0.5)? >= profile_only.alpha(0.5)?);
    assert!(combined.alpha(0.5)? >= rdp_only.alpha(0.5)?);

    let profile_only = PrivacyGuarantee::from_profile(profile);
    let failing_rdp =
        profile_only.with_renyiDP(|_| fallible!(FailedFunction, "RDP failed"), 0.0)?;
    assert_eq!(failing_rdp.delta(1.0)?, 0.2);
    Ok(())
}

#[test]
fn test_zcdp_representation_queries_and_source_delta() -> Fallible<()> {
    let exact = PrivacyGuarantee::new().with_zCDP(0.5, 0.0)?;
    let approximate = PrivacyGuarantee::new().with_zCDP(0.5, 0.1)?;

    assert_eq!(exact.delta(f64::INFINITY)?, 0.0);
    assert_eq!(approximate.delta(f64::INFINITY)?, 0.1);
    assert!(approximate.epsilon(0.1f64.next_down())?.is_infinite());
    assert!(approximate.beta(0.5)? >= 0.0);
    assert!(approximate.alpha(0.5)? >= 0.0);

    assert!(PrivacyGuarantee::new().with_zCDP(-0.0, 0.0).is_err());
    assert!(PrivacyGuarantee::new().with_zCDP(0.5, -0.0).is_err());
    assert!(PrivacyGuarantee::new().with_zCDP(f64::NAN, 0.0).is_err());
    Ok(())
}

#[test]
fn test_zcdp_and_rdp_are_independent_representations() -> Fallible<()> {
    let guarantee = PrivacyGuarantee::new()
        .with_zCDP(0.0, 0.2)?
        .with_renyiDP_trusted(|_| Ok(0.0), 0.7)?;

    // Both representations are queried; zCDP is not silently replaced by RDP.
    assert_eq!(guarantee.delta(f64::INFINITY)?, 0.2);
    assert_eq!(guarantee.epsilon(0.2)?, 0.0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_zcdp_beats_intentionally_loose_native_rdp() -> Fallible<()> {
    // zCDP rho=.1 embeds as the all-orders curve .1*alpha. This is a valid
    // all-orders RDP fact, but intentionally looser than that induced curve.
    let loose = PrivacyGuarantee::new().with_renyiDP(|alpha| Ok(10.0 * alpha), 0.0)?;
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.1, 0.0)?;
    let combined = loose.clone().with_zCDP(0.1, 0.0)?;

    let loose_delta = loose.delta(1.0)?;
    let zcdp_delta = zcdp.delta(1.0)?;
    let combined_delta = combined.delta(1.0)?;
    assert!(zcdp_delta < loose_delta);
    assert_eq!(combined_delta, zcdp_delta);
    let loose_epsilon = loose.epsilon(0.1)?;
    let zcdp_epsilon = zcdp.epsilon(0.1)?;
    let combined_epsilon = combined.epsilon(0.1)?;
    assert!(zcdp_epsilon < loose_epsilon);
    assert_eq!(combined_epsilon, zcdp_epsilon);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_native_rdp_beats_independently_looser_zcdp() -> Fallible<()> {
    // The native .01*alpha curve is tighter than the .1*alpha curve induced
    // by the independently stored zCDP fact.
    let native = PrivacyGuarantee::new().with_renyiDP(|alpha| Ok(0.01 * alpha), 0.0)?;
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.1, 0.0)?;
    let combined = native.clone().with_zCDP(0.1, 0.0)?;

    let native_delta = native.delta(1.0)?;
    let zcdp_delta = zcdp.delta(1.0)?;
    let combined_delta = combined.delta(1.0)?;
    assert!(native_delta < zcdp_delta);
    assert_eq!(combined_delta, native_delta);
    let native_epsilon = native.epsilon(0.1)?;
    let zcdp_epsilon = zcdp.epsilon(0.1)?;
    let combined_epsilon = combined.epsilon(0.1)?;
    assert!(native_epsilon < zcdp_epsilon);
    assert_eq!(combined_epsilon, native_epsilon);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_zcdp_does_not_refine_native_rdp_storage() -> Fallible<()> {
    let guarantee = PrivacyGuarantee::new()
        .with_renyiDP(|alpha| Ok(10.0 * alpha), 0.0)?
        .with_zCDP(0.1, 0.0)?;
    let representation = guarantee.renyi.as_ref().expect("native RDP stored");
    assert_eq!((representation.curve)(2.0)?, 20.0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_high_order_delta_search() -> Fallible<()> {
    let guarantee = PrivacyGuarantee::new().with_renyiDP(|_| Ok(1.0), 0.0)?;
    let delta = guarantee.delta(1.0)?;

    // The order-1024 result is around 1e-3; high-order probing should find
    // the substantially tighter backend-supported bound.
    assert!(delta < 1e-12, "delta={delta}");
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_delta_search_stops_before_unevaluable_order() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = calls.clone();
    let guarantee = PrivacyGuarantee::new().with_renyiDP(
        move |order| {
            callback_calls.fetch_add(1, Ordering::Relaxed);
            if order > 1_000.0 {
                fallible!(FailedFunction, "RDP order is unevaluable")
            } else {
                Ok(1.0)
            }
        },
        0.0,
    )?;

    let delta = guarantee.delta(1.0)?;
    assert!(delta.is_finite());
    assert!(calls.load(Ordering::Relaxed) > 0);
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_nonzero_curve_exercises_all_queries() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = calls.clone();
    let guarantee = PrivacyGuarantee::new().with_renyiDP(
        move |order| {
            callback_calls.fetch_add(1, Ordering::Relaxed);
            Ok(0.01 + 0.05 * order)
        },
        0.0,
    )?;

    let delta = guarantee.delta(1.0)?;
    let epsilon = guarantee.epsilon(1.0)?;
    let beta = guarantee.beta(1.0)?;
    let alpha = guarantee.alpha(1.0)?;
    for value in [delta, epsilon, beta, alpha] {
        assert!(!value.is_nan());
        assert!(value >= 0.0);
    }
    assert!(calls.load(Ordering::Relaxed) > 0);
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
