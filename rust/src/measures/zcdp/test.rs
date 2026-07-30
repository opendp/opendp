use super::*;

use crate::error::Fallible;

#[test]
fn test_zcdp_delta_rounding_regression() -> Fallible<()> {
    assert_eq!(zcdp_delta(0.05, 0.25)?, 8.17727531036956e-2);
    assert_eq!(zcdp_delta(5e-30, 1e-13)?, 2.619676329730073e-234);
    Ok(())
}

#[test]
fn test_edge_cases() -> Fallible<()> {
    assert!(zcdp_delta(-0., 0.).is_err());
    assert!(zcdp_delta(0., -0.).is_err());

    assert_eq!(zcdp_delta(0., 0.)?, 0.);

    let delta = zcdp_delta(0.5, 0.)?;
    assert!((delta - 0.5588356393474351).abs() < 1e-14);
    assert_eq!(zcdp_epsilon(0.5, delta)?, 0.0);

    assert!(zcdp_delta(0.1, 0.1)? > 0.);
    assert_eq!(zcdp_delta(0.1, f64::INFINITY)?, 0.);
    assert_eq!(zcdp_delta(f64::INFINITY, 1.)?, 1.0);
    assert!(zcdp_delta(f64::NAN, 1.).is_err());
    assert!(zcdp_delta(1., f64::NAN).is_err());

    Ok(())
}

#[test]
fn test_delta_is_bounded_and_monotone() -> Fallible<()> {
    for rho in [f64::from_bits(1), 1e-12, 0.01, 0.5, 10., 1e100] {
        let mut previous = 1.0;
        for eps in [0., 1e-12, 0.01, 0.1, 1., 10., 100., 1e100] {
            let delta = zcdp_delta(rho, eps)?;
            assert!(delta > 0.0 && delta <= 1.0, "rho={rho}, eps={eps}");
            assert!(
                delta <= previous,
                "delta increased at rho={rho}, eps={eps}: {delta} > {previous}"
            );
            previous = delta;
        }
    }
    Ok(())
}

#[test]
fn test_tighter_than_classic_zcdp_bound() -> Fallible<()> {
    for rho in [1e-6_f64, 0.01, 0.5, 10.] {
        for eps in [rho, rho + 0.1, rho + 1., rho + 10.] {
            let classic = (-(eps - rho).powi(2) / (4. * rho)).exp();
            let delta = zcdp_delta(rho, eps)?;
            if classic == 0.0 {
                assert_eq!(delta, f64::from_bits(1));
                continue;
            }
            assert!(
                delta <= classic * (1. + 1e-14),
                "rho={rho}, eps={eps}: {delta} > {classic}"
            );
        }
    }
    Ok(())
}

#[test]
fn test_underflow_remains_conservative() -> Fallible<()> {
    assert_eq!(zcdp_delta(1e-6, 1e100)?, f64::from_bits(1));
    Ok(())
}
