use crate::{combinators::{make_fixed_approxDP_to_approxDP, make_pureDP_to_fixed_approxDP, make_zCDP_to_approxDP}, domains::AtomDomain, measurements::{make_scalar_float_gaussian, make_scalar_float_laplace}, metrics::AbsoluteDistance};

use super::*;


#[test]
fn test_smd_curve_epsilon() -> Fallible<()> {

    let meas_laplace = make_scalar_float_laplace(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::default(),
        1.0,
        None,
    )?;
    let meas_laplace_approxDP = make_fixed_approxDP_to_approxDP(make_pureDP_to_fixed_approxDP(meas_laplace)?)?;
    let smd_curve_laplace = meas_laplace_approxDP.map(&1.0)?;

    assert!(smd_curve_laplace.epsilon(0.0)? >= 1.0);
    assert!(smd_curve_laplace.delta(smd_curve_laplace.epsilon(0.0)?)? == 0.0);
    assert!(smd_curve_laplace.epsilon(1.0)? == 0.0);

    let meas_gaussian = make_scalar_float_gaussian(
        AtomDomain::<f64>::default(),
        AbsoluteDistance::default(),
        1.0,
        None
    )?;

    let meas_gaussian_approxDP = make_zCDP_to_approxDP(meas_gaussian)?;
    let smd_curve_gaussian = meas_gaussian_approxDP.map(&1.0)?;
    println!("{}", smd_curve_gaussian.epsilon(0.0)?);

    assert!(smd_curve_gaussian.epsilon(0.0)? == f64::INFINITY);
    assert!(smd_curve_gaussian.epsilon(1.0)? == 0.0);
    assert!(smd_curve_gaussian.delta(smd_curve_gaussian.epsilon(1.0)?)? <= 1.0);
    assert!(smd_curve_gaussian.delta(smd_curve_gaussian.epsilon(2.0)?)? <= 2.0);

    Ok(())
}

#[test]
fn test_beta() -> Fallible<()> {
    let pure_dp_privacy_profile_delta = move |epsilon: f64| {
        let pure_epsilon = 1.0;
        if epsilon >= pure_epsilon { // eps >= 1.0
            return Ok(0.0);
        }

        if epsilon <= (pure_epsilon.exp() - 1.0).ln() { // eps <= 0.54132...
            return Ok(1.0)
        }

        Ok(pure_epsilon.exp() - epsilon.exp())
    };
    let smd_curve = SMDCurve::new(move |epsilon| pure_dp_privacy_profile_delta(epsilon));

    //println!("{}", pure_dp_privacy_profile_delta(0.9999999999999999)?);
    //println!("{}", smd_curve.delta(0.9999999999999999)?);
    
    println!("{}", smd_curve.epsilon(0.0)?);
    println!("{}", smd_curve.epsilon(1.0)?);
    println!("{}", smd_curve.beta(0.0)?);

    let betas = (0..100)
        .map(|i| i as f64 / 100.0)
        .map(|a| smd_curve.beta(a))
        .collect::<Fallible<Vec<f64>>>()?;

    println!("{betas:?}");

    Ok(())
}



#[test]
fn test_all() -> Fallible<()> {
    let pure_dp_privacy_profile_delta = move |epsilon: f64| {
        let pure_epsilon = 1.0;
        if epsilon > pure_epsilon {
            return Ok(0.0);
        }

        if epsilon < (pure_epsilon.exp() - 1.0).ln() {
            return Ok(1.0)
        }

        Ok(pure_epsilon.exp() - epsilon.exp())
    };
    let smd_curve = SMDCurve::new(move |epsilon| pure_dp_privacy_profile_delta(epsilon));

    // Posterior
    let posterior_curve = smd_curve.get_posterior_curve(0.5);
    /*println!("posterior(prior=0.5, alpha=0) = {}", posterior_curve(0.0)?);
    println!(
        "posterior(prior=0.5, alpha=0.5) = {}",
        posterior_curve(0.5)?
    );
    println!("posterior(prior=0.5, alpha=1) = {}", posterior_curve(1.0)?);

    // Relative risk

    let relative_risk_curve = smd_curve.get_relative_risk_curve(0.5);
    println!(
        "relative_risk(prior=0.5, alpha=0) = {}",
        relative_risk_curve(0.0)?
    );
    println!(
        "relative_risk(prior=0.5, alpha=0.5) = {}",
        relative_risk_curve(0.5)?
    );
    println!(
        "relative_risk(prior=0.5, alpha=1) = {}",
        relative_risk_curve(1.0)?
    );*/

    let posts = (0..100)
        .map(|i| i as f64 / 100.0)
        .map(|a| posterior_curve(a))
        .collect::<Fallible<Vec<f64>>>()?;

    println!("{posts:?}");
    /*/
    let rrisks = (0..100)
        .map(|i| i as f64 / 100.0)
        .map(|a| relative_risk_curve(a))
        .collect::<Fallible<Vec<f64>>>()?;

    println!("{rrisks:?}");*/

    Ok(())
}
