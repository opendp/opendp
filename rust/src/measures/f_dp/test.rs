use core::f64;

use crate::{
    combinators::make_zCDP_to_approxDP, domains::AtomDomain, measurements::make_gaussian,
    metrics::AbsoluteDistance,
};

use super::*;

fn meiser_pure_to_approx(epsilon: f64) -> f64 {
    let fixed_epsilon = 1f64;
    (fixed_epsilon.exp() - epsilon.exp()).clamp(0.0, 1.0)
}

#[test]
fn test_privacy_profile() -> Fallible<()> {
    let profile = PrivacyProfile::new(move |epsilon| Ok(meiser_pure_to_approx(epsilon)));
    assert_eq!(profile.epsilon(0.0)?, 0.9999999999999999);
    assert_eq!(profile.delta(1.0)?, 0.0);

    let epsilons = (0..100)
        .map(|i| i as f64 / 100.0)
        .map(|a| profile.epsilon(a))
        .collect::<Fallible<Vec<f64>>>()?;

    epsilons.windows(2).for_each(|w| {
        assert!(w[0] >= w[1]);
    });

    // any epsilon >= 1.0 should return 0.0 delta
    assert_eq!(profile.delta(1.0)?, 0.0);
    assert_eq!(profile.delta(2.0)?, 0.0);

    // delta of more than 1.0 should return an error
    assert!(profile.epsilon(2.0).is_err());

    // roundtrip
    assert_eq!(profile.delta(profile.epsilon(0.0)?)?, 0.0);

    Ok(())
}

#[test]
fn test_beta() -> Fallible<()> {
    let profile = PrivacyProfile::new(move |epsilon| Ok(meiser_pure_to_approx(epsilon)));

    // any type-1 error of 0.0 should return 1.0 type-2 error
    assert_eq!(profile.beta(0.0)?, 1.0);

    let betas = (0..=100)
        .map(|i| i as f64 / 100.0)
        .map(|a| profile.beta(a))
        .collect::<Fallible<Vec<f64>>>()?;

    betas.windows(2).for_each(|w| {
        assert!(w[0] >= w[1]);
    });

    Ok(())
}

#[test]
fn test_beta_gaussian_profile_nonzero_scale() -> Fallible<()> {
    let m_gauss = make_zCDP_to_approxDP(make_gaussian(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        4.0,
        None,
    )?)?;

    let profile = m_gauss.map(&1.0)?;
    assert_eq!(profile.beta(0.0)?, 1.0);
    assert_eq!(profile.beta(f64::from_bits(1))?, 1.0); // smallest positive subnormal
    assert_eq!(profile.beta(f64::MIN_POSITIVE)?, 1.0); // smallest positive normal
    assert_eq!(profile.beta(1e-3)?, 0.9975453589292126);
    assert_eq!(profile.beta(0.5)?, 0.36104607898079);
    assert_eq!(
        profile.beta(f64::from_bits(1f64.to_bits() - 1))?,
        7.074940540780417e-18
    );
    assert_eq!(profile.beta(1.0)?, 0.0);

    let betas = (0..=100)
        .map(|i| i as f64 / 100.0)
        .map(|a| profile.beta(a))
        .collect::<Fallible<Vec<f64>>>()?;

    // some numerical instability in the first 20 values
    betas.windows(2).for_each(|w| {
        assert!(w[0] >= w[1]);
    });
    Ok(())
}

#[test]
fn test_beta_gaussian_profile_zero_scale() -> Fallible<()> {
    let m_gauss = make_zCDP_to_approxDP(make_gaussian(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        0.0,
        None,
    )?)?;

    let profile = m_gauss.map(&1.0)?;
    assert_eq!(profile.beta(0.0)?, 1.0);
    assert_eq!(profile.beta(0.1)?, 0.0);

    Ok(())
}

#[test]
fn test_relative_risk() -> Fallible<()> {
    let profile = PrivacyProfile::new(move |epsilon| Ok(meiser_pure_to_approx(epsilon)));
    let relative_risk_curve = profile.relative_risk_curve(0.5)?;
    assert!(relative_risk_curve(0.0)?.is_nan());
    assert_eq!(relative_risk_curve(0.5)?, 1.2401563852036805);
    assert_eq!(relative_risk_curve(1.0)?, 1.0);

    let rrisks = (1..=100)
        .map(|i| i as f64 / 100.0)
        .map(|a| relative_risk_curve(a))
        .collect::<Fallible<Vec<f64>>>()?;

    // some numerical instability in the first 20 values
    rrisks.windows(2).skip(20).for_each(|w| {
        assert!(w[0] >= w[1]);
    });

    let risk = profile.relative_risk_curve(0.0)?;
    assert!(risk(-1.0).is_err());
    assert!(risk(0.0)?.is_nan());
    assert_eq!(risk(0.5)?, 1.632120558828558);
    assert_eq!(risk(1.0)?, 1.0);
    assert!(risk(2.0).is_err());

    let risk = profile.relative_risk_curve(1.0)?;
    assert!(risk(-1.0).is_err());
    assert!(risk(0.0)?.is_nan());
    assert_eq!(risk(0.5)?, 1.0);
    assert_eq!(risk(1.0)?, 1.0);
    assert!(risk(2.0).is_err());

    Ok(())
}

#[test]
fn test_posterior_relative_risk_gaussian_profile_nonzero_scale() -> Fallible<()> {
    let m_gauss = make_zCDP_to_approxDP(make_gaussian(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        4.0,
        None,
    )?)?;

    let profile = m_gauss.map(&1.0)?;
    let posterior = profile.posterior_curve(0.2)?;
    assert_eq!(posterior(0.5)?, 0.24212394006956922);

    let relative_risk = profile.relative_risk_curve(0.2)?;
    assert_eq!(relative_risk(0.5)?, 1.210619700347846);

    Ok(())
}

#[test]
fn test_posterior() -> Fallible<()> {
    let profile = PrivacyProfile::new(move |epsilon| Ok(meiser_pure_to_approx(epsilon)));
    let posterior_curve = profile.posterior_curve(0.5)?;
    assert!(posterior_curve(0.0)?.is_nan());
    assert_eq!(posterior_curve(0.5)?, 0.6200781926018403);
    assert_eq!(posterior_curve(1.0)?, 0.5);

    let posts = (1..=100)
        .map(|i| i as f64 / 100.0)
        .map(|a| posterior_curve(a))
        .collect::<Fallible<Vec<f64>>>()?;

    // some numerical instability in the first 20 values
    posts.windows(2).skip(20).for_each(|w| {
        assert!(w[0] >= w[1]);
    });

    let posterior = profile.posterior_curve(0.0)?;
    assert!(posterior(-1.0).is_err());
    assert!(posterior(0.0)?.is_nan());
    assert_eq!(posterior(0.5)?, 0.0);
    assert_eq!(posterior(1.0)?, 0.0);
    assert!(posterior(2.0).is_err());

    let posterior = profile.posterior_curve(1.0)?;
    assert!(posterior(-1.0).is_err());
    assert!(posterior(0.0)?.is_nan());
    assert_eq!(posterior(0.5)?, 1.0);
    assert_eq!(posterior(1.0)?, 1.0);
    assert!(posterior(2.0).is_err());

    Ok(())
}
