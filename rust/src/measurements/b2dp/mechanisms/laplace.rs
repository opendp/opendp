//! Implements an approximation of the Laplace mechanims via the base-2 exponential mechanism.
//! Note: this does incur the factor of 2 penalty on privacy loss. Consider `sample_within_bounds`
//! or `noisy_threshold` as alternatives.

use crate::error::Fallible;
use crate::measurements::b2dp::mechanisms::exponential::{
    exponential_mechanism, ExponentialOptions,
};
use crate::measurements::b2dp::utilities::params::Eta;
use rug::rand::ThreadRandGen;

/// Implements clamped Laplace mechanism via the base-2 exponential mechanism
/// Outputs an `f64` in the range `[lower_bound, upper_bound]` of the form
/// `lower_bound + gamma*i`.
/// ## Arguments
///   * `eta`: the base-2 privacy parameter
///   * `lower_bound`: the lower bound of the range for the Laplace mechanism
///   * `upper_bound`: the upper bound of the range for the Laplace mechanism
///   * `target`: the target value (value to approximate)
///   * `gamma`: the granularity of the outcome space
///   * `rng`: a random number generator
///
/// ## Returns
/// Returns an `f64` within the specified bounds.
///
/// ## Errors
/// Returns Err if any of the parameters are configured incorrectly or if inexact arithmetic
/// occurs.
///
/// ## Example
/// Sample from the Laplace mechanism targeting `0.0` clamped to the range `[-10,10]` with
/// granularity `0.25`.
/// ```
/// use b2dp::{mechanisms::laplace::clamped_laplace_mechanism, Eta, GeneratorOpenSSL};
///
/// let target = 0.0;
/// let eta = Eta::new(1,1,1).unwrap();
/// let lower_bound = -10.0;
/// let upper_bound = 10.0;
/// let gamma = 0.25;
/// let mut rng = GeneratorOpenDP::default();
/// let sample = clamped_laplace_mechanism(eta, lower_bound, upper_bound, target, gamma, &mut rng, Default::default()).unwrap();
/// ```
#[deprecated(note = "Please use `sample_within_bounds` as an alternative to this method.")]
pub fn clamped_laplace_mechanism<R: ThreadRandGen>(
    eta: Eta,
    lower_bound: f64,
    upper_bound: f64,
    target: f64,
    gamma: f64,
    rng: &mut R,
    options: ExponentialOptions,
) -> Fallible<f64> {
    // Check Parameters
    eta.check()?;
    if lower_bound >= upper_bound {
        return fallible!(FailedFunction, "lower_bound must be smaller than upper_bound");
    }

    // Clamp the target
    let mut clamp_target = target;
    if clamp_target > upper_bound {
        clamp_target = upper_bound;
    }
    if clamp_target < lower_bound {
        clamp_target = upper_bound;
    }

    // Generate exponential mechanism parameters
    let mut outcomes: Vec<f64> = Vec::new();

    let mut x = lower_bound;

    while x <= upper_bound {
        outcomes.push(x);
        x += gamma;
    }
    let max_outcomes = outcomes.len();

    let laplace_utility = |x: &f64| -> f64 { (clamp_target - *x).abs() };

    let utility_max = upper_bound - lower_bound;

    let sample = exponential_mechanism(
        eta,
        &outcomes,
        laplace_utility,
        0,
        utility_max as u32,
        max_outcomes as u32,
        rng,
        options,
    )
    .unwrap();

    Ok(*sample)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::samplers::GeneratorOpenDP;
    #[test]
    fn test_laplace_mechanism_basic() {
        let eta = Eta::new(1, 1, 1).unwrap();
        let mut rng = GeneratorOpenDP::default();

        #[allow(deprecated)]
        let sample =
            clamped_laplace_mechanism(eta, -10.0, 10.0, 1.0, 0.25, &mut rng, Default::default());
        assert!(sample.is_ok());
    }
}
