use crate::{measures::PrivacyGuarantee, measures::zcdp::zcdp_epsilon};

use crate::{
    combinators::make_approximate, domains::AtomDomain, measurements::make_gaussian,
    metrics::AbsoluteDistance,
};

use super::*;

#[test]
fn test_zCDP_to_approxDP_nontrivial() -> Fallible<()> {
    let d_in = 1.0;
    let scale = 4.0;
    let profile = make_zCDP_to_approxDP(make_gaussian::<_, _, zCDP>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        scale,
        None,
    )?)?
    .map(&d_in)?;
    let rho = (d_in / scale).powi(2) / 2.0;
    let direct = PrivacyGuarantee::new().with_zCDP(rho, 0.0)?;

    assert_eq!(profile.epsilon(0.)?, f64::INFINITY);
    assert_eq!(profile.epsilon(1e-3)?, direct.epsilon(1e-3)?);
    assert_eq!(
        profile.delta(0.6880024554878085)?,
        direct.delta(0.6880024554878085)?
    );

    // Compare the two independently optimized directions to within one ulp.
    let epsilon = zcdp_epsilon(rho, 1e-3)?;
    let profile_epsilon = profile.epsilon(1e-3)?;
    assert!((profile_epsilon - epsilon).abs() <= f64::EPSILON * epsilon);
    assert_eq!(profile.epsilon(1.0)?, 0.);

    // using reverse map to check correctness
    let zero_epsilon_delta = profile.delta(0.)?;
    assert_eq!(zcdp_epsilon(rho, zero_epsilon_delta)?, 0.0);
    assert_eq!(profile.delta(0.)?, zero_epsilon_delta);
    let delta = profile.delta(0.6880024554878085)?;
    assert!(delta >= 1e-3);
    assert!((delta - 1e-3) <= 4. * f64::EPSILON * 1e-3);
    Ok(())
}

#[test]
fn test_zCDP_to_approxDP_insensitive() -> Fallible<()> {
    let profile = make_zCDP_to_approxDP(make_gaussian::<_, _, zCDP>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        4.,
        None,
    )?)?
    .map(&0.0)?;

    assert_eq!(profile.epsilon(0.0)?, 0.0);

    assert!(profile.epsilon(-0.0).is_err());
    assert!(profile.delta(-0.0).is_err());
    Ok(())
}

#[test]
fn test_zCDP_to_approxDP_nonprivate() -> Fallible<()> {
    let profile = make_zCDP_to_approxDP(make_gaussian::<_, _, zCDP>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        0.,
        None,
    )?)?
    .map(&1.0)?;

    assert_eq!(profile.epsilon(0.0)?, f64::INFINITY);
    assert_eq!(profile.epsilon(0.1)?, f64::INFINITY);
    assert_eq!(profile.delta(0.0)?, 1.0);
    assert_eq!(profile.delta(0.1)?, 1.0);
    Ok(())
}

#[test]
fn test_zCDP_to_approxDP_insensitive_nonprivate() -> Fallible<()> {
    let profile = make_zCDP_to_approxDP(make_gaussian::<_, _, zCDP>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        0.,
        None,
    )?)?
    .map(&0.0)?;

    assert_eq!(profile.epsilon(0.0)?, 0.0);
    assert_eq!(profile.epsilon(0.1)?, 0.0);
    assert_eq!(profile.delta(0.0)?, 0.0);
    assert_eq!(profile.delta(0.1)?, 0.0);
    Ok(())
}

#[test]
fn test_approx_zCDP_to_approx_approxDP() -> Fallible<()> {
    let m_zcdp = make_gaussian(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        1.,
        None,
    )?;

    let m_azcdp = make_approximate(m_zcdp)?;
    let m_adp = make_zCDP_to_approxDP(m_azcdp)?;

    let (curve, delta) = m_adp.map(&1.0)?;
    assert_eq!(delta, 0.0);

    let epsilon = curve.epsilon(1e-7)?;

    // when scale is 1 and sensitivity is 1, then rho = (d_in / scale)^2 / 2 = 0.5
    let expected_epsilon = zcdp_epsilon(0.5, 1e-7)?;
    assert_eq!(epsilon, expected_epsilon);

    Ok(())
}
