use cdp_delta::test::cdp_epsilon;

use crate::{domains::AtomDomain, measurements::make_gaussian, metrics::AbsoluteDistance};

use super::*;

#[test]
fn test_zCDP_to_approxDP_nontrivial() -> Fallible<()> {
    let d_in = 1.0;
    let scale = 4.0;
    let profile = make_zCDP_to_approxDP(make_gaussian(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::default(),
        scale,
        None,
    )?)?
    .map(&d_in)?;
    let rho = (d_in / scale).powi(2) / 2.0;

    assert_eq!(profile.epsilon(0.)?, f64::INFINITY);

    // using reverse map to check correctness
    // implementation of reverse map is slightly looser by 1 ulp due to numerical imprecision
    assert_eq!(cdp_epsilon(rho, 1e-3)?, 0.6880024554878086);
    assert_eq!(profile.epsilon(1e-3)?, 0.6880024554878085);
    assert_eq!(profile.epsilon(1.0)?, 0.);

    // using reverse map to check correctness
    assert_eq!(cdp_epsilon(rho, 0.1508457845622862)?, 0.0);
    assert_eq!(profile.delta(0.)?, 0.1508457845622862);
    assert_eq!(profile.delta(0.6880024554878085)?, 1e-3);
    Ok(())
}

#[test]
fn test_zCDP_to_approxDP_insensitive() -> Fallible<()> {
    let profile = make_zCDP_to_approxDP(make_gaussian(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::default(),
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
    let profile = make_zCDP_to_approxDP(make_gaussian(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::default(),
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
    let profile = make_zCDP_to_approxDP(make_gaussian(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::default(),
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
