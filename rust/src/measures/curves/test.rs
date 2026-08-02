use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{
    error::Fallible, measures::PrivacyCurve, measures::privacy_profile::log_to_delta_upper,
};

#[test]
fn test_privacy_profile_from_approxdp_pairs() -> Fallible<()> {
    let pairs = vec![(0.0, 1.0), (0.1, 1e-3), (0.5, 1e-7), (1.0, 0.0)];
    let profile = PrivacyCurve::new().with_approxDP(pairs)?;

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
    let profile = PrivacyCurve::new().with_approxDP(vec![(1.0, 1e-7)])?;

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

    let curve = PrivacyCurve::new().with_approxDP(vec![(epsilon, delta)])?;
    assert!(curve.delta(epsilon)? <= delta);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_delta_slack_shifts_profile_and_inverse() -> Fallible<()> {
    let curve = PrivacyCurve::new()
        .with_log_profile(|eps| Ok(-eps))?
        .with_delta_slack(1e-6)?;

    assert_eq!(curve.delta(f64::INFINITY)?, 1e-6);
    assert!(curve.epsilon(0.5e-6)?.is_infinite());

    let epsilon = curve.epsilon((-2.0f64).exp() + 1e-6)?;
    assert!((epsilon - 2.0).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_delta_slack_matches_equivalent_approxdp_curve() -> Fallible<()> {
    let epsilon = 1.0;
    let delta_slack = 0.1;

    let with_slack = PrivacyCurve::new()
        .with_approxDP(vec![(epsilon, 0.0)])?
        .with_delta_slack(delta_slack)?;
    let direct = PrivacyCurve::new().with_approxDP(vec![(epsilon, delta_slack)])?;

    for alpha in [0.0, 0.1, 0.25, 0.5, 0.9, 1.0] {
        assert!((with_slack.beta(alpha)? - direct.beta(alpha)?).abs() < 1e-12);
    }

    assert_eq!(with_slack.delta(0.5)?, direct.delta(0.5)?);
    assert!(
        with_slack.delta(1.0)? >= direct.delta(1.0)?
            && with_slack.delta(1.0)? <= direct.delta(1.0)?.next_up()
    );
    assert_eq!(
        with_slack.epsilon(delta_slack)?,
        direct.epsilon(delta_slack)?
    );
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_inversion_conservative_endpoints() -> Fallible<()> {
    let cutoff = 1.0f64;
    let profile = PrivacyCurve::new().with_log_profile(move |epsilon| {
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
        PrivacyCurve::new().with_tradeoff(|alpha| Ok(if alpha < 0.5 { 0.75 } else { 0.25 }))?;
    let alpha = tradeoff.alpha(0.25)?;
    assert_eq!(alpha, 0.5f64.next_down());
    assert!(tradeoff.beta(alpha)? > 0.25);
    Ok(())
}

#[test]
fn test_input_validation() {
    assert!(
        PrivacyCurve::new()
            .with_approxDP(vec![(-0.0, 0.0)])
            .is_err()
    );
    assert!(
        PrivacyCurve::new()
            .with_approxDP(vec![(0.0, -0.0)])
            .is_err()
    );
    assert!(PrivacyCurve::new().with_delta_slack(-0.0).is_err());

    let curve = PrivacyCurve::new().with_approxDP(vec![(0.0, 0.0)]).unwrap();
    assert!(curve.delta(-0.0).is_err());
    assert!(curve.beta(-0.0).is_err());
    assert!(curve.epsilon(-0.0).is_err());
    assert!(curve.alpha(-0.0).is_err());

    assert!(
        PrivacyCurve::new()
            .with_approxDP(vec![(0.0, 0.1), (1.0, 0.2)])
            .is_err()
    );
}

#[test]
fn test_composition_identity_and_directed_sums() -> Fallible<()> {
    let identity = PrivacyCurve::compose(vec![])?;
    assert_eq!(identity.delta(0.0)?, 0.0);
    assert_eq!(identity.beta(0.25)?, 0.75);

    let first = PrivacyCurve::new().with_approxDP(vec![(0.1, 0.2)])?;
    let second = PrivacyCurve::new().with_approxDP(vec![(0.2, 0.3)])?;
    let composed = PrivacyCurve::compose(vec![first, second])?;

    let epsilon = composed.epsilon(0.5)?;
    assert!(epsilon >= 0.1 + 0.2);
    assert!(composed.delta(epsilon)? >= 0.5);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_ordinary_profile_plateau_inversion() -> Fallible<()> {
    let curve =
        PrivacyCurve::new().with_profile(|epsilon| Ok(if epsilon >= 2.0 { 0.1 } else { 1.0 }))?;

    assert_eq!(curve.epsilon(0.1)?, 2.0);
    assert_eq!(curve.epsilon(0.1f64.next_down())?, f64::INFINITY);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_unreachable_ordinary_profile_delta_inverts_to_infinity() -> Fallible<()> {
    let curve = PrivacyCurve::new().with_profile(|_| Ok(0.5))?;
    assert_eq!(curve.epsilon(0.25)?, f64::INFINITY);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_log_profile_generic_inversion() -> Fallible<()> {
    let curve = PrivacyCurve::new().with_log_profile(|epsilon| Ok(-epsilon))?;
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
    let curve = PrivacyCurve::new().with_log_profile_with_epsilon(
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

#[test]
fn test_zcdp_curve_uses_shared_profile_conversion() -> Fallible<()> {
    let rho = 0.5;
    let epsilon = 1.0;
    let curve = PrivacyCurve::new().with_zCDP(rho)?;
    assert_eq!(
        curve.delta(epsilon)?,
        crate::measures::zcdp_delta(rho, epsilon)?
    );
    assert!((0.0..=1.0).contains(&curve.beta(0.5)?));
    Ok(())
}

#[test]
fn test_zcdp_and_rdp_composition_is_conservative() -> Fallible<()> {
    assert!(PrivacyCurve::new().with_zCDP(-0.0).is_err());

    let first = PrivacyCurve::new().with_zCDP(0.1)?;
    let second = PrivacyCurve::new().with_zCDP(0.2)?;
    let composed = PrivacyCurve::compose(vec![first, second])?;
    assert!(composed.zcdp.unwrap() >= 0.1 + 0.2);

    let rdp = PrivacyCurve::new().with_renyiDP_trusted(|_| Ok(0.1))?;
    let zcdp = PrivacyCurve::new().with_zCDP(0.2)?;
    let composed = PrivacyCurve::compose(vec![rdp, zcdp])?;
    let epsilon = composed.renyi_dp.unwrap()(2.0)?;
    assert!(epsilon >= 0.1 + 2.0 * 0.2);

    let non_private = PrivacyCurve::new().with_zCDP(f64::INFINITY)?;
    let composed = PrivacyCurve::compose(vec![non_private])?;
    assert_eq!(composed.zcdp, Some(f64::INFINITY));
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_renyi_curve_queries() -> Fallible<()> {
    let curve = PrivacyCurve::new().with_renyiDP(|alpha| Ok(alpha * 0.25))?;
    assert!((0.0..=1.0).contains(&curve.delta(1.0)?));
    assert!((0.0..=1.0).contains(&curve.beta(0.5)?));
    Ok(())
}

#[test]
fn test_renyi_error_propagation() -> Fallible<()> {
    let curve = PrivacyCurve::new()
        .with_renyiDP_trusted(|_| fallible!(FailedFunction, "test RDP failure"))?;
    assert!(curve.delta(1.0).is_err());
    assert!(curve.epsilon(1e-3).is_err());
    Ok(())
}
