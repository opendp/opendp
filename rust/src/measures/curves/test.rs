#[cfg(feature = "honest-but-curious")]
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{
    error::Fallible,
    measures::{PrivacyGuarantee, curves::logspace::log_to_delta_upper},
};

#[test]
fn test_privacy_profile_from_approxdp_pairs() -> Fallible<()> {
    let pairs = vec![(0.0, 1.0), (0.1, 1e-3), (0.5, 1e-7), (1.0, 0.0)];
    let profile = PrivacyGuarantee::new().with_approxDP(pairs)?;

    // Test exact points
    assert_eq!(profile.delta(0.0)?, 1.0);
    assert_eq!(profile.delta(1.0)?, 0.0);

    // Test conservative stairstep behavior
    let mid = profile.delta(0.05)?;
    assert_eq!(mid, 1.0);
    assert_eq!(profile.delta(0.3)?, 1e-3);

    Ok(())
}

#[test]
fn test_privacy_profile_from_single_approxdp_pair() -> Fallible<()> {
    let profile = PrivacyGuarantee::new().with_approxDP(vec![(1.0, 1e-7)])?;

    assert_eq!(profile.delta(0.5)?, 1.0);
    assert_eq!(profile.delta(1.0)?, 1e-7);
    assert_eq!(profile.delta(2.0)?, 1e-7);

    let beta = profile.beta(0.25)?;
    assert!(0.0 < beta && beta < 1.0);

    Ok(())
}

#[test]
fn test_tradeoff_curve_from_approxdp_satisfies_profile() -> Fallible<()> {
    let epsilon = 0.5;
    let delta = 1e-6;

    let curve = PrivacyGuarantee::new().with_approxDP(vec![(epsilon, delta)])?;
    assert!(curve.delta(epsilon)? <= delta);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_public_tradeoff_endpoints() -> Fallible<()> {
    let approx = PrivacyGuarantee::new().with_approxDP(vec![(1.0, 0.1)])?;
    let beta_zero = approx.beta(0.0)?;
    assert!(beta_zero < 1.0);
    assert!(beta_zero > 0.89);

    let pure = PrivacyGuarantee::new().with_approxDP(vec![(1.0, 0.0)])?;
    assert_eq!(pure.beta(0.0)?, 1.0);

    let callback =
        PrivacyGuarantee::new().with_tradeoff(|alpha| Ok((0.75_f64 - 1.5 * alpha).max(0.0)))?;
    assert_eq!(callback.beta(0.0)?, 0.75);
    let alpha_zero = callback.alpha(0.0)?;
    assert_eq!(alpha_zero, 0.5_f64.next_down());
    assert!(callback.beta(alpha_zero)? > 0.0);
    assert_eq!(callback.beta(alpha_zero.next_up())?, 0.0);
    let delta_infinity = callback.delta(f64::INFINITY)?;
    assert!(delta_infinity >= 0.5);
    assert!(delta_infinity <= 0.5_f64.next_up());
    assert!(callback.delta(1000.0)? >= delta_infinity);
    assert!(callback.delta(f64::MAX)? >= delta_infinity);

    let no_zero_before_one =
        PrivacyGuarantee::new().with_tradeoff(|alpha| Ok(0.5 * (1.0 - alpha)))?;
    let no_zero_delta_infinity = no_zero_before_one.delta(f64::INFINITY)?;
    assert!(no_zero_delta_infinity >= 0.5);
    assert!(no_zero_delta_infinity <= 0.5_f64.next_up());
    assert!(no_zero_before_one.delta(1000.0)? >= no_zero_delta_infinity);

    let already_zero = PrivacyGuarantee::new().with_tradeoff(|_| Ok(0.0))?;
    assert!(already_zero.delta(f64::INFINITY)? >= 1.0);

    let symmetric =
        PrivacyGuarantee::new().with_symmetric_tradeoff(|alpha| Ok((0.5_f64 - alpha).max(0.0)))?;
    assert_eq!(symmetric.beta(0.0)?, 0.5);
    assert_eq!(symmetric.alpha(0.0)?, 0.5);
    let symmetric_delta_infinity = symmetric.delta(f64::INFINITY)?;
    assert!(symmetric_delta_infinity >= 0.5);
    assert!(symmetric_delta_infinity <= 0.5_f64.next_up());
    assert!(symmetric.delta(1000.0)? >= symmetric_delta_infinity);

    let profile = PrivacyGuarantee::new()
        .with_profile(|epsilon| Ok(if epsilon.is_infinite() { 0.2 } else { 0.8 }))?;
    let profile_beta_zero = profile.beta(0.0)?;
    assert!(profile_beta_zero < 0.81);
    assert!(profile_beta_zero > 0.79);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_tradeoff_infinity_endpoint_evaluates_endpoints_once() -> Fallible<()> {
    let zero_calls = Arc::new(AtomicUsize::new(0));
    let one_calls = Arc::new(AtomicUsize::new(0));
    let zero_calls_ = zero_calls.clone();
    let one_calls_ = one_calls.clone();
    let curve = PrivacyGuarantee::new().with_tradeoff(move |alpha| {
        if alpha == 0.0 {
            zero_calls_.fetch_add(1, Ordering::Relaxed);
        } else if alpha == 1.0 {
            one_calls_.fetch_add(1, Ordering::Relaxed);
        }
        Ok((0.75_f64 - 1.5 * alpha).max(0.0))
    })?;

    curve.delta(f64::INFINITY)?;
    assert_eq!(zero_calls.load(Ordering::Relaxed), 1);
    assert_eq!(one_calls.load(Ordering::Relaxed), 1);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_tradeoff_callback_errors_propagate() -> Fallible<()> {
    let at_zero = PrivacyGuarantee::new().with_tradeoff(|alpha| {
        if alpha == 0.0 {
            fallible!(FailedFunction, "callback failed at alpha zero")
        } else {
            Ok(0.0)
        }
    })?;
    assert!(at_zero.delta(f64::INFINITY).is_err());

    let during_optimization = PrivacyGuarantee::new().with_tradeoff(|alpha| {
        if alpha > 0.0 && alpha < 1.0 {
            fallible!(FailedFunction, "callback failed during optimization")
        } else {
            Ok(if alpha == 1.0 { 0.0 } else { 0.4 })
        }
    })?;
    assert!(during_optimization.delta(1.0).is_err());

    let invalid_during_optimization = PrivacyGuarantee::new().with_tradeoff(|alpha| {
        if alpha > 0.0 && alpha < 1.0 {
            Ok(2.0)
        } else {
            Ok(if alpha == 1.0 { 0.0 } else { 0.4 })
        }
    })?;
    assert!(invalid_during_optimization.delta(1.0).is_err());
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_public_tradeoff_generalized_inverse_endpoints() -> Fallible<()> {
    let curve = PrivacyGuarantee::new().with_tradeoff(|alpha| Ok((0.75_f64 - alpha).max(0.0)))?;

    let alpha = curve.alpha(0.25)?;
    assert_eq!(alpha, 0.5_f64.next_down());
    assert!(curve.beta(alpha)? > 0.25);
    assert!(curve.beta(alpha.next_up())? <= 0.25);

    let alpha = curve.alpha(0.0)?;
    assert_eq!(alpha, 0.75_f64.next_down());
    assert!(curve.beta(alpha)? > 0.0);
    assert_eq!(curve.beta(alpha.next_up())?, 0.0);

    let approx = PrivacyGuarantee::new().with_approxDP(vec![(1.0, 0.1)])?;
    let alpha_zero = approx.alpha(0.0)?;
    assert!(alpha_zero < 0.91);
    assert!(alpha_zero > 0.89);
    assert!(approx.beta(alpha_zero.next_up())? <= 0.0);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_inversion_conservative_endpoints() -> Fallible<()> {
    let cutoff = 1.0f64;
    let profile = PrivacyGuarantee::new().with_log_profile(move |epsilon| {
        Ok(if epsilon < cutoff {
            0.5f64.ln()
        } else {
            0.25f64.ln()
        })
    })?;
    let target_delta = profile.delta(cutoff)?;
    assert_eq!(profile.epsilon(target_delta)?, cutoff);
    assert!(profile.delta(cutoff.next_down())? > target_delta);

    let tradeoff =
        PrivacyGuarantee::new().with_tradeoff(|alpha| Ok((0.75_f64 - alpha).max(0.0)))?;
    let alpha = tradeoff.alpha(0.25)?;
    assert_eq!(alpha, 0.5f64.next_down());
    assert!(tradeoff.beta(alpha)? > 0.25);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_callback_delta_validation_and_single_evaluation() -> Fallible<()> {
    let cases = [
        (0.0, false),
        (-0.0, false),
        (1.0, false),
        (f64::from_bits(1), false),
        (-f64::from_bits(1), true),
        (1.0 + f64::EPSILON, true),
        (f64::NAN, true),
    ];

    for (value, invalid) in cases {
        let curve = PrivacyGuarantee::new().with_profile(move |_| Ok(value))?;
        assert_eq!(curve.delta(0.0).is_err(), invalid);
        if !invalid && value == 0.0 {
            assert_eq!(curve.delta(0.0)?.to_bits(), 0.0f64.to_bits());
        }
    }

    let calls = Arc::new(AtomicUsize::new(0));
    let calls_ = calls.clone();
    let curve = PrivacyGuarantee::new().with_profile(move |_| {
        calls_.fetch_add(1, Ordering::Relaxed);
        Ok(0.25)
    })?;
    assert_eq!(curve.delta(0.0)?, 0.25f64.next_up());
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    let log_calls = Arc::new(AtomicUsize::new(0));
    let log_calls_ = log_calls.clone();
    let log_curve = PrivacyGuarantee::new().with_log_profile(move |_| {
        log_calls_.fetch_add(1, Ordering::Relaxed);
        Ok(-1.0)
    })?;
    log_curve.delta(0.0)?;
    assert_eq!(log_calls.load(Ordering::Relaxed), 1);

    let invalid = PrivacyGuarantee::new().with_profile(|_| Ok(2.0))?;
    assert!(invalid.epsilon(0.5).is_err());
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_callback_validation_through_beta() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_ = calls.clone();
    let curve = PrivacyGuarantee::new().with_log_profile(move |_| {
        calls_.fetch_add(1, Ordering::Relaxed);
        Ok(1.0)
    })?;

    assert!(curve.beta(0.5).is_err());
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_callback_log_delta_validation() -> Fallible<()> {
    let cases = [
        (f64::NEG_INFINITY, false),
        (0.0, false),
        (-0.0, false),
        (f64::from_bits(1), true),
        (f64::NAN, true),
        (f64::INFINITY, true),
    ];

    for (value, invalid) in cases {
        let curve = PrivacyGuarantee::new().with_log_profile(move |_| Ok(value))?;
        assert_eq!(curve.delta(0.0).is_err(), invalid);
    }

    let invalid = PrivacyGuarantee::new().with_log_profile(|_| Ok(1.0))?;
    assert!(invalid.epsilon(0.5).is_err());
    Ok(())
}

#[test]
fn test_input_validation() -> Fallible<()> {
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(-0.0, 0.0)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(f64::NAN, 0.0)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, f64::NAN)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, -0.1)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 1.1)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, -0.1), (0.0, 0.2)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 1.0), (1.0, f64::NAN)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, -0.0)])
            .is_err()
    );
    let curve = PrivacyGuarantee::new()
        .with_approxDP(vec![(0.0, 0.0)])
        .unwrap();
    assert!(curve.delta(-0.0).is_err());
    assert!(curve.beta(-0.0).is_err());
    assert!(curve.epsilon(-0.0).is_err());
    assert!(curve.alpha(-0.0).is_err());

    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.1), (1.0, 0.2)])
            .is_err()
    );

    // Every point is validated before duplicate and structural checks.
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, f64::NAN), (0.0, 0.1)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.1), (1.0, f64::NAN)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.1), (-1.0, 0.2)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.1), (0.0, 2.0)])
            .is_err()
    );

    // Canonical input is strict: duplicate, plateau, and dominated points are
    // rejected rather than repaired.
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.1), (0.0, 0.1)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.2), (0.0, 0.1)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.1), (1.0, 0.1)])
            .is_err()
    );
    assert!(
        PrivacyGuarantee::new()
            .with_approxDP(vec![(0.0, 0.1), (1.0, 0.2), (2.0, 0.3)])
            .is_err()
    );

    let sorted = PrivacyGuarantee::new().with_approxDP(vec![(0.0, 0.1), (1.0, 0.01)])?;
    let unsorted = PrivacyGuarantee::new().with_approxDP(vec![(1.0, 0.01), (0.0, 0.1)])?;
    assert_eq!(unsorted.delta(0.0)?, sorted.delta(0.0)?);
    assert_eq!(unsorted.delta(1.0)?, sorted.delta(1.0)?);
    assert_eq!(unsorted.epsilon(0.01)?, sorted.epsilon(0.01)?);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_ordinary_profile_plateau_inversion() -> Fallible<()> {
    let curve = PrivacyGuarantee::new()
        .with_profile(|epsilon| Ok(if epsilon >= 2.0 { 0.1 } else { 1.0 }))?;

    assert_eq!(curve.epsilon(0.1)?, 2.0);
    assert_eq!(curve.epsilon(0.1f64.next_down())?, f64::INFINITY);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_profile_zero_endpoints() -> Fallible<()> {
    let ordinary = PrivacyGuarantee::new()
        .with_profile(|epsilon| Ok(if epsilon < 1.0 { 1.0 } else { 0.0 }))?;
    assert_eq!(ordinary.epsilon(0.0)?, 1.0);
    assert_eq!(ordinary.delta(f64::INFINITY)?, 0.0);

    let log = PrivacyGuarantee::new().with_log_profile(|epsilon| {
        Ok(if epsilon < 1.0 {
            0.0
        } else {
            f64::NEG_INFINITY
        })
    })?;
    assert_eq!(log.epsilon(0.0)?, 1.0);
    assert_eq!(log.delta(f64::INFINITY)?, 0.0);

    let asymptotic = PrivacyGuarantee::new().with_log_profile(|epsilon| Ok(-epsilon))?;
    assert_eq!(asymptotic.epsilon(0.0)?, f64::INFINITY);

    let direct = PrivacyGuarantee::new().with_log_profile_with_epsilon(
        |_| Ok(f64::NEG_INFINITY),
        |delta| Ok(if delta == 0.0 { 2.0 } else { 0.0 }),
    )?;
    assert_eq!(direct.epsilon(0.0)?, 2.0);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_delta_at_positive_infinity_uses_profile_representation() -> Fallible<()> {
    let ordinary = PrivacyGuarantee::new()
        .with_profile(|epsilon| Ok(if epsilon.is_infinite() { 0.25 } else { 0.5 }))?;
    assert_eq!(ordinary.delta(f64::INFINITY)?, 0.25f64.next_up());

    let log = PrivacyGuarantee::new().with_log_profile(|epsilon| {
        Ok(if epsilon.is_infinite() {
            0.125f64.ln()
        } else {
            0.25f64.ln()
        })
    })?;
    assert!(log.delta(f64::INFINITY)? >= 0.125);

    let staircase = PrivacyGuarantee::new().with_approxDP(vec![(1.0, 1e-3)])?;
    assert_eq!(staircase.delta(f64::INFINITY)?, 1e-3);

    let ending_zero = PrivacyGuarantee::new().with_approxDP(vec![(1.0, 0.0)])?;
    assert_eq!(ending_zero.delta(f64::INFINITY)?, 0.0);

    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_profile_inversion_consumes_terminal_range_errors() -> Fallible<()> {
    let below = PrivacyGuarantee::new().with_log_profile(|epsilon| {
        if epsilon >= 2.0 {
            fallible!(NumericRangeBelow, "log(delta) is below the numeric range")
        } else {
            Ok(-epsilon)
        }
    })?;
    let epsilon = below.epsilon((-1.5f64).exp())?;
    assert!(epsilon.is_finite());
    assert!(epsilon >= 1.5);

    let above = PrivacyGuarantee::new().with_log_profile(|epsilon| {
        if epsilon < 1.0 {
            fallible!(NumericRangeAbove, "log(delta) is above the numeric range")
        } else {
            Ok(-epsilon)
        }
    })?;
    let epsilon = above.epsilon((-1.5f64).exp())?;
    assert!(epsilon.is_finite());
    assert!(epsilon >= 1.5);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_unreachable_ordinary_profile_delta_inverts_to_infinity() -> Fallible<()> {
    let curve = PrivacyGuarantee::new().with_profile(|_| Ok(0.5))?;
    assert_eq!(curve.epsilon(0.25)?, f64::INFINITY);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_log_profile_generic_inversion() -> Fallible<()> {
    let curve = PrivacyGuarantee::new().with_log_profile(|epsilon| Ok(-epsilon))?;
    assert!(curve.delta(2.0)? >= (-2.0f64).exp());
    assert!(curve.epsilon((-2.0f64).exp())? >= 2.0);
    assert_eq!(curve.epsilon(0.0)?, f64::INFINITY);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_certified_profile_inverse_bypasses_forward_search() -> Fallible<()> {
    let forward_calls = Arc::new(AtomicUsize::new(0));
    let forward_calls_ = forward_calls.clone();
    let curve = PrivacyGuarantee::new().with_log_profile_with_epsilon(
        move |epsilon| {
            forward_calls_.fetch_add(1, Ordering::Relaxed);
            Ok(-epsilon)
        },
        |_delta| Ok(3.0),
    )?;

    assert_eq!(curve.epsilon(1e-6)?, 3.0);
    assert_eq!(forward_calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[test]
fn test_subnormal_log_to_delta_conversion() -> Fallible<()> {
    let minimum = f64::from_bits(1);
    assert_eq!(log_to_delta_upper(minimum.ln())?, f64::from_bits(2));
    assert_eq!(log_to_delta_upper(minimum.ln().next_down())?, minimum);
    Ok(())
}
