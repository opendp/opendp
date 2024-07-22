// TODO:
// . Mike TODO at some point: remove SMDCurve in favor of function (separate)

// . implement β(α) curve conversion -> draft
// . implement posterior curve -> done
// . relative risk curve -> done
// . FFI for above
// . visualizations in Python
// . adjust internal implementation to use δ(ε) curve, add ε(δ) helper via bs
// . parameterize gaussian mechanism
// . combinator for making a trivial δ(ε) from (ε, δ) (already have a conversion from ε, to (ε, 0)) 
use crate::{
    core::Function,
    error::Fallible,
    measures::SMDCurve
};


fn profile_to_tradeoff(
    curve: SMDCurve<f64>,
) -> Fallible<Function<f64, f64>> {

    Ok(Function::new_fallible(move |alpha: &f64| -> Fallible<f64> {
        if *alpha < 0.0 || *alpha > 1.0 {
            return fallible!(FailedMap, "alpha must be in [0, 1]");
        }

        let beta = find_best_supporting_beta(&curve, *alpha)?;

        Ok(beta)
    }))
}

/// Finds the best supporting tradeoff curve and returns the highest
/// beta for a given a privacy curve and alpha
///
/// # Arguments:
/// * `curve` - Privacy curve
/// * `alpha` - must be within [0, 1]
fn find_best_supporting_beta(curve: &SMDCurve<f64>, alpha: f64) -> Fallible<f64> {
    // Ternary search for delta that maximizes beta in the interval [0, 1]
    // Could be improved with golden search algorithm or setting
    // delta_mid_left to (delta_right - delta_left)/2 - very_small_value
    let mut delta_left = 0.0;
    let mut delta_right = 1.0;
    loop {
        let third = (delta_right - delta_left) / 3.0;
        let delta_mid_left = delta_left + third;
        let delta_mid_right = delta_right - third;

        // Stopping criteria
        if delta_left == delta_mid_left && delta_right == delta_mid_right {
            // TODO what if there are still values between delta_left and delta_right? try them all?
            let epsilon = curve.epsilon(&delta_left)?; // Arbitrary between left and right.
            let beta = support_tradeoff(alpha, epsilon, delta_left);
            return Ok(beta);
        }
        
        let epsilon_mid_left = curve.epsilon(&delta_mid_left)?;
        let epsilon_mid_right = curve.epsilon(&delta_mid_right)?;
        let beta_mid_left = support_tradeoff(alpha, epsilon_mid_left, delta_mid_left);
        let beta_mid_right = support_tradeoff(alpha, epsilon_mid_right, delta_mid_right);

        if beta_mid_left > beta_mid_right {
            delta_right = delta_mid_right;
        } else if beta_mid_left < beta_mid_right {
            delta_left = delta_mid_left;
        } else { // beta_mid_left == beta_mid_right
            delta_left = delta_mid_left;
            delta_right = delta_mid_right;
        }
    }
}


/// Computes the β parameter associated with an (ε, δ) linear supporting curve at α
/// 
/// # Arguments
/// * `alpha`- must be within [0, 1]
/// * `epsilon`- must be non-negative
/// * `delta`- must be within [0, 1]
fn support_tradeoff(alpha: f64, epsilon: f64, delta: f64) -> f64 {
    let left = 1.0 - delta - (epsilon.exp() * alpha);
    let right = (-epsilon).exp() * (1.0 - delta - alpha);

    left.max(right).max(0.0)
}


/// Computes the posterior curve given tradeoff curve and attacker's prior probability
/// in a membership attack.
/// 
/// The returned Function takes an alpha value and returns the attacker's posterior.
/// TODO does the posterior only take values in (0, 1] instead of [0, 1]?
/// 
/// # Arguments
/// * `tradeoff_curve` - Tradeoff curve for the measurement
/// * `prior` - Attacker's prior probability.
pub fn get_posterior_curve(tradeoff_curve: Function<f64, f64>, prior: f64) -> Fallible<Function<f64, f64>> {
    Ok(Function::new_fallible(move |alpha: &f64| {
        let beta = tradeoff_curve.eval(alpha)?;
        Ok((prior * (1.0 - beta)) / ((1.0 - prior)* (*alpha) + prior * (1.0 - beta)))
    }))
}


/// Computes the relative risk curve given tradeoff curve and attacker's prior probability
/// in a membership attack.
/// 
/// The returned Function takes an alpha value and returns the relative risk.
/// TODO does the relative risk only take values in (0, 1] instead of [0, 1]?
/// 
/// # Arguments
/// * `tradeoff_curve` - Tradeoff curve for the measurement
/// * `prior` - Attacker's prior probability.
pub fn get_relative_risk_curve(tradeoff_curve: Function<f64, f64>, prior: f64) -> Fallible<Function<f64, f64>> {
    
    Ok(Function::new_fallible(move |alpha: &f64| {
        let beta = tradeoff_curve.eval(alpha)?;
        Ok((1.0 - beta) / ((1.0 - prior)* (*alpha) + prior * (1.0 - beta)))
    }))
}


#[test]
fn test_all() -> Fallible<()> {
    let epsilon: f64 = 1.0;
    let pure_dp_privacy_profile = move |delta: f64| {
        if delta == 0.0 {
            return Ok(epsilon);
        }
        
        Ok((epsilon.exp() - delta).ln())
    };
    let smd_curve = SMDCurve::new(move |&delta| pure_dp_privacy_profile(delta));
    
    // Tradeoff
    let tradeoff_curve = profile_to_tradeoff(smd_curve).unwrap();
    println!("tradeoff(0) = {}", tradeoff_curve.eval(&0.0).unwrap());
    println!("tradeoff(0.27) = {}", tradeoff_curve.eval(&0.27).unwrap());
    println!("tradeoff(1) = {}", tradeoff_curve.eval(&1.0).unwrap());
    
    // Posterior
    let posterior_curve: Function<f64, f64> = get_posterior_curve(tradeoff_curve.clone(), 0.5).unwrap();
    println!("posterior(prior=0.5, alpha=0) = {}", posterior_curve.eval(&0.0).unwrap());
    println!("posterior(prior=0.5, alpha=0.5) = {}", posterior_curve.eval(&0.5).unwrap());
    println!("posterior(prior=0.5, alpha=1) = {}", posterior_curve.eval(&0.5).unwrap());

    // Relative risk
    
    let relative_risk_curve: Function<f64, f64> = get_relative_risk_curve(tradeoff_curve.clone(), 0.5).unwrap();
    println!("relative_risk(prior=0.5, alpha=0) = {}", relative_risk_curve.eval(&0.0).unwrap());
    println!("relative_risk(prior=0.5, alpha=0.5) = {}", relative_risk_curve.eval(&0.5).unwrap());
    println!("relative_risk(prior=0.5, alpha=1) = {}", relative_risk_curve.eval(&0.5).unwrap());

    let alphas: Vec<f64> = (0..1000).map(|i| i as f64 / 1000.0).collect();
    for a in &alphas {
        println!("{}", relative_risk_curve.eval(&a).unwrap());
    }


    /*
    Questions:
    - Rust closure "move"
    - Why need to clone tradeoff curve, isn't it copied with the "move" keyword? -> do the clone
    - what delta "precision" is enough, arbitrary choice, leave option to user?

    TODOs:
    - arbitrary choice of "delta precision"
    - replace ternary search? seems relatively quick as-is.
    - tests
    - function names, etc..
    */

    Ok(())

}
// fn exhaustive_search() {
//     // Function not strictly convex, walk again until we find two different betas
//     let mut step = third;
//     loop {
//         step /= 2.0;

//         if step <= f64::powf(2.0, -20.0) {
//             // all beta values between eps_left and eps_right are equal, return highest of eps_right and eps_left
//             let beta_left = support_tradeoff(alpha, eps_left, self.delta(eps_left)?);
//             let beta_right = support_tradeoff(alpha, eps_right, self.delta(eps_right)?);

//             return Ok(beta_left.max(beta_right));
//         }

//         eps_mid_left = eps_left + step;
//         beta_mid_left =
//             support_tradeoff(alpha, eps_mid_left, self.delta(eps_mid_left)?);

//         // Try all values between eps_mid_left and eps_right in step increment
//         eps_mid_right = eps_mid_left;

//         loop {
//             eps_mid_right = eps_mid_right + step;
//             if eps_mid_right >= eps_right {
//                 break;
//             } // did not find two different values.

//             beta_mid_right =
//                 support_tradeoff(alpha, eps_mid_right, self.delta(eps_mid_right)?);

//             if beta_mid_right != beta_mid_left {
//                 break;
//             }
//         }

//         if beta_mid_left > beta_mid_right {
//             eps_right = eps_mid_right;
//             break;
//         } else if beta_mid_left < beta_mid_right {
//             eps_left = eps_mid_left;
//             break;
//         }
//     }
// }
