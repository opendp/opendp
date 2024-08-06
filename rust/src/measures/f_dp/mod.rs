use core::f64;

use dashu::{
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
    rbig,
};
use num::{One, Zero};
use opendp_derive::proven;

use crate::{error::Fallible, measures::PrivacyProfile, traits::InfCast};

#[cfg(all(test, feature = "contrib"))]
mod test;

impl PrivacyProfile {
    /// Returns the largest Type II error (beta) for a given Type I error (alpha).
    ///
    /// The returned beta is a (slightly) conservative underestimate.
    ///
    /// Each pareto-optimal pair of (epsilon, delta) defines a linear supporting curve.
    /// The beta function is the maximum of the linear supporting curves at a given alpha.
    /// Since the betas corresponding to the linear supporting curves are convex and unimodal over epsilon,
    /// a ternary search is used to find the best linear supporting tradeoff curve on the privacy profile.
    ///
    /// # Arguments
    /// * `alpha` - Type I error. Must be within [0, 1]
    pub fn beta(&self, alpha: f64) -> Fallible<f64> {
        if alpha < 0.0 || alpha > 1.0 {
            return fallible!(FailedMap, "alpha must be in [0, 1]");
        }
        if alpha.is_zero() {
            return Ok(1.0);
        }
        if alpha.is_one() {
            return Ok(0.0);
        }

        // Ternary search for epsilon in [epsilon(1-alpha), epsilon(delta=0)] that maximizes beta.
        // In other words, find max of beta(alpha) function.
        // We assume the beta function is concave. If not, return value will be no greater than true beta

        // Reduce the search space of epsilons to make the curve strictly convex
        let mut eps_left = self.epsilon_unchecked(1.0 - alpha)?; // for delta >= 1 - alpha, support curve will be zero anyway.

        if eps_left.is_infinite() {
            return Ok(0.0);
        }

        if eps_left.is_nan() {
            return fallible!(FailedMap, "epsilon(1-alpha) is NaN");
        }
        // Find an upper bound for valid epsilons.
        // This avoids starting epsilon in a regime where the curve is not computable.
        let mut delta_right = f64::MIN_POSITIVE;
        while self.epsilon_unchecked(delta_right).is_err() {
            delta_right *= 2.0;
        }
        let mut eps_right = self.epsilon_unchecked(delta_right)?.min(f64::MAX); // epsilon(delta) returns smallest epsilon st. delta(epsilon) ~= 0.0, fine since f(e1, d) > f(e2, d) if e1 < e2

        loop {
            let third = (eps_right - eps_left) / 3.0;
            let eps_mid_left = eps_left + third;
            let eps_mid_right = eps_right - third;

            // Stopping criteria
            if eps_left == eps_mid_left && eps_right == eps_mid_right {
                let delta = self.delta(eps_right)?; // Arbitrary between left and right.

                // This is the only place where conservative arithmetic is needed.
                // If the search doesn't find the "optimal" curve due to numerical issues,
                // then beta will be overestimated, which is fine.
                let beta = approximate_to_tradeoff((eps_right, delta))?.0(RBig::try_from(alpha)?);
                return f64::inf_cast(beta).map_err(|_| err!(FailedMap, "beta is not finite"));
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

    /// Returns a relative risk curve for a hypothetical adversary who has a prior
    /// on the probability of an individual being a member of the dataset.
    ///
    /// The relative risk curve computes the relative increase in the strength
    /// of the adversary's prior at a given Type I error (alpha).
    ///
    /// # Arguments
    /// * `prior` - Attacker's prior membership probability. Must be within [0, 1]
    pub fn relative_risk_curve(
        &self,
        prior: f64,
    ) -> Fallible<impl Fn(f64) -> Fallible<f64> + Clone> {
        let curve = self.clone();
        if prior < 0.0 || prior > 1.0 {
            return fallible!(FailedMap, "prior must be in [0, 1]");
        }
        Ok(move |alpha: f64| {
            let beta = curve.beta(alpha)?;
            Ok((1.0 - beta) / ((1.0 - prior) * alpha + prior * (1.0 - beta)))
        })
    }

    /// Returns a posterior curve for a hypothetical adversary who has a prior
    /// on the probability of an individual being a member of the dataset.
    ///
    /// The posterior curve computes the adversary's posterior knowledge
    /// about the probability of an individual being a member of the dataset
    /// at a given Type I error (alpha).
    ///
    /// # Arguments
    /// * `prior` - Attacker's prior membership probability. Must be within [0, 1]
    pub fn posterior_curve(&self, prior: f64) -> Fallible<impl Fn(f64) -> Fallible<f64> + Clone> {
        let rel_risk_curve = self.relative_risk_curve(prior)?;
        Ok(move |alpha| Ok(prior * rel_risk_curve(alpha)?))
    }
}

/// Computes the beta parameter associated with an (ε, δ) linear supporting curve at alpha.
///
/// Helper function for a fast parameter search.
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

#[proven]
/// # Proof Definition
/// Given epsilon and delta, return the corresponding f-DP tradeoff curve
/// with conservative arithmetic,
/// as well as the fixed point `c` where `c = f(c)`.
/// Returns an error if epsilon or delta are invalid.
pub(crate) fn approximate_to_tradeoff(
    (epsilon, delta): (f64, f64),
) -> Fallible<(impl Fn(RBig) -> RBig + 'static + Clone + Send + Sync, RBig)> {
    if epsilon.is_sign_negative() || epsilon.is_zero() {
        return fallible!(
            MakeMeasurement,
            "epsilon ({epsilon}) must not be positive (greater than zero)"
        );
    }
    if !(0.0..=1.0).contains(&delta) {
        return fallible!(MakeMeasurement, "delta ({delta}) must be within [0, 1]");
    }

    let epsilon = FBig::<Down>::try_from(epsilon)?;
    let delta = RBig::try_from(delta)?;

    // exp(ε)
    let exp_eps = epsilon.clone().with_rounding::<Down>().exp();
    let exp_eps = RBig::try_from(exp_eps)?;

    // exp(-ε)
    let exp_neg_eps = (-epsilon).with_rounding::<Up>().exp();
    let exp_neg_eps = RBig::try_from(exp_neg_eps)?;

    //              = (1 - δ) / (1 + exp(ε))
    let fixed_point = (rbig!(1) - &delta) / (rbig!(1) + &exp_eps);

    // greater than 1/2 means the tradeoff curve is greater than 1 - x, which is invalid
    // exactly 1 / 2 means perfect privacy, and results in an infinite loop when sampling "infinite" noise
    if fixed_point >= rbig!(1 / 2) {
        return fallible!(
            MakeMeasurement,
            "fixed-point of the f-DP tradeoff curve must be less than 1/2. This indicates that your privacy parameters are too small."
        );
    }

    let tradeoff = move |alpha: RBig| {
        let t1 = rbig!(1) - &delta - &exp_eps * &alpha;
        let t2 = &exp_neg_eps * (rbig!(1) - &delta - alpha);
        t1.max(t2).max(rbig!(0))
    };
    Ok((tradeoff, fixed_point))
}
