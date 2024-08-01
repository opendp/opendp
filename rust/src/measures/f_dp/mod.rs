use crate::{
    error::Fallible,
    measures::SMDCurve,
    traits::{InfExp, InfMul, InfSub},
};

impl SMDCurve {
    /// Finds the best supporting tradeoff curve and returns the highest
    /// beta for a given a privacy curve and alpha
    ///
    /// # Arguments:
    /// * `curve` - Privacy curve
    /// * `alpha` - must be within [0, 1]
    pub fn beta(&self, alpha: f64) -> Fallible<f64> {
        if alpha < 0.0 || alpha > 1.0 {
            return fallible!(FailedMap, "alpha must be in [0, 1]");
        }

        // Ternary search for epsilon in [epsilon(1-alpha), epsilon(delta=0)] that maximizes beta.
        // In other words, find max of beta(epsilon, alpha) function.
        // We assume the beta function is concave. If not, return value will be no greater than true beta

        // Could potentially be improved with golden search algorithm

        // Reduce the search space of epsilons to make the curve strictly convex
        let mut eps_left = self.epsilon_unchecked(1.0 - alpha)?; // for delta >= 1 - alpha, support curve will be zero anyway.
        let mut eps_right = self.epsilon_unchecked(0.0)?.min(f64::MAX); // epsilon(delta) returns smallest epsilon st. delta(epsilon) ~= 0.0, fine since f(e1, d) > f(e2, d) if e1 < e2

        loop {
            let third = (eps_right - eps_left) / 3.0;
            let eps_mid_left = eps_left + third;
            let eps_mid_right = eps_right - third;

            // Stopping criteria
            if eps_left == eps_mid_left && eps_right == eps_mid_right {
                let delta = self.delta(eps_right)?; // Arbitrary between left and right.
                return conservative_support_tradeoff(alpha, eps_right, delta);
            }

            let delta_mid_left = self.delta(eps_mid_left)?;
            let delta_mid_right = self.delta(eps_mid_right)?;
            let beta_mid_left = support_tradeoff(alpha, eps_mid_left, delta_mid_left);
            let beta_mid_right = support_tradeoff(alpha, eps_mid_right, delta_mid_right);

            if beta_mid_left > beta_mid_right {
                eps_right = eps_mid_right;
            } else {
                eps_left = eps_mid_left;
            }
        }
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
    pub fn get_relative_risk_curve(&self, prior: f64) -> impl Fn(f64) -> Fallible<f64> + Clone {
        let curve = self.clone();
        move |alpha: f64| {
            let beta = curve.beta(alpha)?;
            Ok((1.0 - beta) / ((1.0 - prior) * alpha + prior * (1.0 - beta)))
        }
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
    pub fn get_relative_risk_curve(&self, prior: f64) -> impl Fn(f64) -> Fallible<f64> + Clone {
        let curve = self.clone();
        move |alpha: f64| {
            let beta = curve.beta(alpha)?;
            Ok((1.0 - beta) / ((1.0 - prior) * alpha + prior * (1.0 - beta)))
        }
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
    pub fn get_posterior_curve(&self, prior: f64) -> impl Fn(f64) -> Fallible<f64> + Clone {
        let rel_risk_curve = self.get_relative_risk_curve(prior);
        move |alpha| Ok(prior * rel_risk_curve(alpha)?)
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

/// Computes the β parameter associated with an (ε, δ) linear supporting curve at α,
/// with conservative arithmetic
///
/// # Arguments
/// * `alpha`- must be within [0, 1]
/// * `epsilon`- must be non-negative
/// * `delta`- must be within [0, 1]
fn conservative_support_tradeoff(alpha: f64, epsilon: f64, delta: f64) -> Fallible<f64> {
    // re-implements support_tradeoff
    let left = (1.0_f64)
        .neg_inf_sub(&delta)?
        .neg_inf_sub(&epsilon.inf_exp()?.inf_mul(&alpha)?)?;
    let right = (-epsilon)
        .neg_inf_exp()?
        .neg_inf_mul(&((1.0).neg_inf_sub(&delta)?).neg_inf_sub(&alpha)?)?;

    Ok(left.max(right).max(0.0))
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
