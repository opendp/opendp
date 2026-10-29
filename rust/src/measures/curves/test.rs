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
    let target = (-0.25f64).exp();
    let epsilon = profile.epsilon(target)?;
    // Invert the supplied f64 bound, including its equality plateau, rather
    // than comparing to the analytic inverse of an unrounded exponential.
    assert!(profile.delta(epsilon)? <= target);
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

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_intermediate_overflow_does_not_establish_comparison() -> Fallible<()> {
    use crate::traits::{CInterval, SInterval};
    for sign in [-1.0, 1.0] {
        // The final negation reverses the intermediate's sign. Software
        // intervals can retain the finite value; native intervals cannot.
        let exact_final = -(SInterval::point(sign * f64::MAX)? * SInterval::point(2.0)?)?;
        assert_eq!(exact_final.is_nonpositive(), sign > 0.0);
        let profile = PrivacyProfile::new(|_| Ok(1.0)).with_log_profile(move |epsilon| {
            if epsilon < 2.0 {
                return Ok(0.0);
            }
            let intermediate = (CInterval::point(sign * f64::MAX)? * CInterval::point(2.0)?)?;
            (-intermediate).upper_f64()
        })?;
        for delta in [0.1, 0.0] {
            assert_eq!(
                profile.epsilon(delta).unwrap_err().variant,
                ErrorVariant::Overflow
            );
        }
        assert_eq!(
            profile.pure_epsilon().unwrap_err().variant,
            ErrorVariant::Overflow
        );
    }
    Ok(())
}

#[test]
fn test_inversion_comparison_endpoints_and_plateaus() -> Fallible<()> {
    let step = |epsilon| {
        Ok(if epsilon < 2.0 {
            0.0
        } else if epsilon < 4.0 {
            -1.0
        } else {
            -2.0
        })
    };
    assert_eq!(invert_decreasing_callback(step, 0.0)?, 0.0);
    assert_eq!(invert_decreasing_callback(step, -1.0)?, 2.0);
    assert_eq!(invert_decreasing_callback(step, -2.0)?, 4.0);
    assert_eq!(
        invert_decreasing_callback(step, -3.0).unwrap_err().variant,
        ErrorVariant::Search
    );

    let overflow = |_| fallible!(Overflow, "unknown intermediate overflow");
    for target in [-1.0, f64::NEG_INFINITY, f64::INFINITY] {
        assert_eq!(
            invert_decreasing_callback(overflow, target)
                .unwrap_err()
                .variant,
            ErrorVariant::Overflow
        );
    }
    assert_eq!(
        PrivacyProfile::new(|_| Ok(0.1)).epsilon(0.0)?,
        f64::INFINITY
    );
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_inversion_preserves_other_numerical_errors() -> Fallible<()> {
    for variant in ["Overflow", "NumericError"] {
        for fail_at_zero in [true, false] {
            let profile = PrivacyProfile::new(|_| Ok(1.0)).with_log_profile(move |epsilon| {
                if fail_at_zero || epsilon >= 2.0 {
                    return Err(crate::error::Error {
                        variant: variant.parse()?,
                        ..err!(FailedMap, "unchanged failure")
                    });
                }
                Ok(0.0)
            })?;
            let error = profile.epsilon(0.1).unwrap_err();
            assert_eq!(error.variant.to_string(), variant);
            assert_eq!(error.message.as_deref(), Some("unchanged failure"));
        }
    }
    Ok(())
}

#[test]
fn test_delta_profile_inversion_does_not_accept_adjacent_larger_delta() -> Fallible<()> {
    let target = 0.1_f64;
    let larger = target.next_up();
    let constant = PrivacyProfile::new(move |_| Ok(larger));
    assert_eq!(constant.epsilon(target)?, f64::INFINITY);
    assert_eq!(constant.epsilon(larger)?, 0.0);
    assert_eq!(constant.delta(f64::INFINITY)?, larger);

    let profile = PrivacyProfile::new(move |epsilon| {
        Ok(if epsilon < 2.0 {
            1.0
        } else if epsilon < 4.0 {
            larger
        } else if epsilon < 6.0 {
            target
        } else {
            0.0
        })
    });
    for (delta, expected_epsilon) in [
        (1.0, 0.0),
        (larger, 2.0),
        (target, 4.0),
        (target.next_down(), 6.0),
        (0.0, 6.0),
    ] {
        let epsilon = profile.epsilon(delta)?;
        assert_eq!(epsilon, expected_epsilon);
        assert!(profile.delta(epsilon)? <= delta);
    }
    assert_eq!(profile.delta(4.0)?, target);
    assert_eq!(profile.delta(f64::INFINITY)?, 0.0);
    for target in [
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        0.5,
        1.0_f64.next_down(),
    ] {
        let larger = target.next_up();
        assert_eq!(
            PrivacyProfile::new(move |_| Ok(larger)).epsilon(target)?,
            f64::INFINITY
        );
        let equal = PrivacyProfile::new(move |_| Ok(target));
        assert_eq!(equal.epsilon(target)?, 0.0);
        assert_eq!(equal.delta(0.0)?, target);
    }
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_log_profile_inversion_never_raises_target() -> Fallible<()> {
    use crate::traits::SInterval;
    let target = 0.1_f64;
    let retained = PrivacyProfile::new(|_| Ok(0.0)).with_profile(move |_| Ok(target))?;
    assert_eq!(retained.epsilon(target)?, 0.0);
    assert_eq!(retained.delta(0.0)?, target);
    let log_upper = delta_to_log_upper_unchecked(target)?;
    // A 256-bit directed reference proves that the rounded-up logarithm
    // represents a strictly larger delta, even though its f64 log matches.
    assert!(SInterval::point(log_upper)?.exp()?.lower() > SInterval::point(target)?.upper());
    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_log_profile(move |_| Ok(log_upper))?;
    assert_eq!(profile.epsilon(target)?, f64::INFINITY);

    // Log-only profiles keep a single conservative target, including exact
    // log-space equality, plateaus, and the mathematical endpoints.
    let log_lower = delta_to_log_lower_unchecked(target)?;
    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_log_profile(move |epsilon| {
        Ok(if epsilon < 2.0 {
            0.0
        } else if epsilon < 4.0 {
            log_upper
        } else if epsilon < 6.0 {
            log_lower
        } else {
            f64::NEG_INFINITY
        })
    })?;
    assert_eq!(profile.epsilon(target)?, 4.0);
    assert_eq!(profile.epsilon(0.0)?, 6.0);
    assert_eq!(profile.epsilon(1.0)?, 0.0);
    assert!(profile.delta(4.0)? <= target);
    assert_eq!(profile.delta(f64::INFINITY)?, 0.0);
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
