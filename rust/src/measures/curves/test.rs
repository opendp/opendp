use crate::{error::Fallible, measures::PrivacyCurve};

#[test]
fn test_privacy_profile_from_approxdp_pairs() -> Fallible<()> {
    let pairs = vec![(0.0, 1.0), (0.1, 1e-3), (0.5, 1e-7), (1.0, 0.0)];
    let profile = PrivacyCurve::new_approxdp(pairs)?;

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
    let profile = PrivacyCurve::new_approxdp(vec![(1.0, 1e-7)])?;

    assert_eq!(profile.delta(0.5)?, 1.0);
    assert_eq!(profile.delta(1.0)?, 1e-7);
    assert_eq!(profile.delta(2.0)?, 1e-7);

    let beta = profile.beta(0.25)?;
    assert!(0.0 < beta && beta < 1.0);

    Ok(())
}

#[test]
fn test_tradeoff_profile_conversions() -> Fallible<()> {
    let curve = PrivacyCurve::new_gdp(1.0)?;

    for alpha in [0.1, 0.5, 0.9] {
        let beta = curve.beta(alpha)?;
        assert!((0.0..=1.0).contains(&beta));
    }
    assert!(curve.beta(0.0)? >= 0.0);

    let epsilon = 1.0;
    // let expected_delta = normal_cdf(-epsilon + 0.5) - epsilon.exp() * normal_cdf(-epsilon - 0.5);
    // computed with an arb oracle
    let expected_delta =
        0.12693673750664394580082962475776688041508065011200132233065575436734191042454632;
    let delta = curve.delta(epsilon)?;
    assert!(delta >= expected_delta);
    assert!(
        (delta - expected_delta).abs() < 1e-15,
        "delta: {}; expected: {}",
        delta,
        expected_delta
    );
    assert!(curve.epsilon(delta)? <= epsilon);
    Ok(())
}

#[test]
fn test_tradeoff_curve_from_approxdp_satisfies_profile() -> Fallible<()> {
    let epsilon = 0.5;
    let delta = 1e-6;

    let curve = PrivacyCurve::new_approxdp(vec![(epsilon, delta)])?;
    assert!(curve.delta(epsilon)? <= delta);
    Ok(())
}
