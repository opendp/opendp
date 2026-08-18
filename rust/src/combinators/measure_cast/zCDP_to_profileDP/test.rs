use crate::measures::zcdp::test::cdp_epsilon;

use crate::{
    combinators::make_approximate, domains::AtomDomain, measurements::make_gaussian,
    measures::zCDP, metrics::AbsoluteDistance,
};

use super::*;

#[test]
fn test_zCDP_to_profileDP_nontrivial() -> Fallible<()> {
    let d_in = 1.0;
    let scale = 4.0;
    let profile = make_zCDP_to_profileDP(make_gaussian(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        scale,
        None,
    )?)?
    .map(&d_in)?;
    let rho = (d_in / scale).powi(2) / 2.0;

    assert_eq!(profile.epsilon(0.)?, f64::INFINITY);

    // using reverse map
    assert_eq!(cdp_epsilon(rho, 1e-3)?, 0.6880024554878086);
    let epsilon = profile.epsilon(1e-3)?;
    assert!(epsilon >= cdp_epsilon(rho, 1e-3)?);
    assert!((epsilon - 0.6880024554878086).abs() < 1e-15);
    assert_eq!(profile.epsilon(1.0)?, 0.);

    assert_eq!(cdp_epsilon(rho, 0.1508457845622862)?, 0.0);
    assert!(profile.delta(0.)? >= 0.1508457845622862);
    assert!(profile.delta(0.6880024554878085)? >= 1e-3);
    Ok(())
}

#[test]
fn test_zCDP_to_profileDP_insensitive() -> Fallible<()> {
    let profile = make_zCDP_to_profileDP(make_gaussian::<_, _, zCDP>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        4.,
        None,
    )?)?
    .map(&0.0)?;

    assert_eq!(profile.epsilon(0.0)?, 0.0);

    assert_eq!(profile.epsilon(-0.0)?, 0.0);
    assert_eq!(profile.delta(-0.0)?, 0.0);
    Ok(())
}

#[test]
fn test_zCDP_to_profileDP_nonprivate() -> Fallible<()> {
    let profile = make_zCDP_to_profileDP(make_gaussian(
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
fn test_zCDP_to_profileDP_insensitive_nonprivate() -> Fallible<()> {
    let profile = make_zCDP_to_profileDP(make_gaussian::<_, _, zCDP>(
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
    let m_adp = make_zCDP_to_profileDP(m_azcdp)?;

    let (curve, delta) = m_adp.map(&1.0)?;
    assert_eq!(delta, 0.0);

    let epsilon = curve.epsilon(1e-7)?;
    let expected_epsilon = cdp_epsilon(0.5, 1e-7)?;
    assert_eq!(epsilon, expected_epsilon);

    Ok(())
}
