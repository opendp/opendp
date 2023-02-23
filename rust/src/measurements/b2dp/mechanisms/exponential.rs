//! Implements the base-2 exponential mechanism.

use crate::error::Fallible;
use crate::measurements::b2dp::utilities::exactarithmetic::{
    normalized_sample, randomized_round, ArithmeticConfig,
};
use crate::measurements::b2dp::utilities::params::Eta;
use rug::{ops::Pow, rand::ThreadRandGen, Float};

/// The exponential mechanism optional parameters.
#[derive(Debug, Clone, Copy)]
pub struct ExponentialOptions {
    /// The minimum number of retries for timing channel prevention
    /// default: `1`
    /// Minimum retries helps to mitigate timing channels in optimized
    /// sampling. The higher the number of retries, the less likely
    /// it is for an adversary to observe useful timing information.
    pub min_retries: u32,

    /// Whether to optimize sampling
    /// default: `false`
    /// Optimized sampling exacerbates timing channels, and it's not
    /// recommended for use in un-trusted settings.
    pub optimized_sample: bool,

    /// Whether to use empirical precision
    /// default: `false`
    /// Determination of safe precision given utility bounds and outcome
    /// set size limits can be done analytically or empirically. Both
    /// are independent of the dataset. Using `empirical_precision = true`
    /// determines the required precision via a set of test calculations.
    /// The timing overhead of these calculations is proportional to the outcome
    /// set size, and the overhead may outweigh any reduction in required
    /// precision.
    pub empirical_precision: bool,
}
impl Default for ExponentialOptions {
    /// Default options for the exponential mechanism
    /// `min_retries = 1`, `optimized_sample = false`, `empirical_precision = false`
    fn default() -> ExponentialOptions {
        ExponentialOptions {
            min_retries: 1,
            optimized_sample: false,
            empirical_precision: false,
        }
    }
}

/// The exponential mechanism configuration. Includes all parameters
/// and information needed to derive the appropriate precision for the
/// mechanism.
#[derive(Debug)]
struct ExponentialConfig {
    /// The privacy parameter
    pub eta: Eta,
    /// The minimum utility (maximum weight)
    pub utility_min: u32,
    /// The maximum utility (minimum weight)
    pub utility_max: u32,
    /// The maximum size of the outcome space
    #[allow(dead_code)]
    pub max_outcomes: u32,
    /// The arithmetic configuration
    arithmetic_config: ArithmeticConfig,
}

// Constructors
impl ExponentialConfig {
    /// Create a new context for the exponential mechanism.
    ///
    /// ## Arguments
    ///   * `eta`: the base-2 privacy parameter
    ///   * `utility_min`: the minimum utility permitted by the mechanism (highest possible weight)
    ///   * `utility_max`: the maximum utility permitted by the mechanism (lowest possible weight)
    ///   * `max_outcomes`: the maximum number of outcomes this instance exponential mechanism permits.
    ///
    /// ## Returns
    /// An `ExponentialConfig` from the specified parameters or an error.
    ///
    /// ## Errors
    /// Returns an error if any of the parameters are mis-specified, or if sufficient precision cannot
    /// be determined.
    pub fn new(
        eta: Eta,
        utility_min: u32,
        utility_max: u32,
        max_outcomes: u32,
        empirical_precision: bool,
        min_retries: u32,
    ) -> Fallible<ExponentialConfig> {
        // Parameter sanity checking
        if utility_min > utility_max {
            return fallible!(FailedFunction, "utility_min must be smaller than utility_max.");
        }
        if max_outcomes == 0 {
            return fallible!(FailedFunction, "Must provide a positive value for max_outcomes.");
        }

        let arithmetic_config = ArithmeticConfig::for_exponential(
            &eta,
            utility_min,
            utility_max,
            max_outcomes,
            empirical_precision,
            min_retries,
        )?;

        // Construct the configuration with the precision we determined above
        let config = ExponentialConfig {
            eta,
            utility_min,
            utility_max,
            max_outcomes,
            arithmetic_config,
        };
        Ok(config)
    }

    /// Wrapper function for `Eta::get_base`. Returns
    /// `eta.get_base()` using the precision specified by
    /// `self.arithmetic_config`.
    pub fn get_base(&self) -> Float {
        self.eta.get_base(self.arithmetic_config.precision).unwrap()
    }
}

/// Implements the base-2 exponential mechanism.
/// Utility convention is to take `-utility(o)`, and `utility_min` is therefore the highest
/// possible weight/maximum probability outcome. This mechanism does not scale based on
/// the sensitivity of the utility function. For a utility function with sensitivity `alpha`,
/// the mechanism is `2*alpha*eta` base-2 DP, and `2*alpha*ln(2)*eta` base-e DP.  
/// **The caller must ensure that `utility_min`, `utility_max`, `max_outcomes`
/// and `outcomes` are determined independently of the `utility` function and any private
/// data.**
///
/// ## Arguments
///   * `eta`: the base-2 privacy parameter
///   * `outcomes`: the set of outcomes the mechanism chooses from
///   * `utility`: utility function operating on elements of `outcomes`. `utility` does not
///                explicitly take a database input, and is expected to have a pointer to the database
///                or access to the private data needed to determine utilities.
///   * `utility_min`: the minimum utility permitted by the mechanism (highest possible weight)
///   * `utility_max`: the maximum utility permitted by the mechanism (lowest possible weight)
///   * `max_outcomes`: the maximum number of outcomes permitted by the mechanism
///   * `rng`: a random number generator
///
/// ## Returns
/// Returns a reference to an element in `outcomes` sampled according to the base-2 exponential
/// mechanism.
///
/// ## Known Timing Channels
/// **This mechanism has known timing channels.** Please see
/// [normalized_sample](../../utilities/exactarithmetic/fn.normalized_sample.html#known-timing-channels).
///
/// ## Errors
/// Returns Err if any of the parameters are configured incorrectly or if inexact arithmetic
/// occurs.
/// ## Example
/// ```
/// use b2dp::{exponential_mechanism, Eta, GeneratorOpenSSL, ExponentialOptions};
///
/// fn util_fn (x: &u32) -> f64 {
///     return ((*x as f64)-0.0).abs();
/// }
/// let eta = Eta::new(1,1,1).unwrap();
/// let utility_min = 0;
/// let utility_max = 10;
/// let max_outcomes = 10;
/// let rng = GeneratorOpenDP::default();
/// let options = ExponentialOptions {min_retries: 1, optimized_sample: true, empirical_precision: false};
/// let outcomes: Vec<u32> = (0..max_outcomes).collect();
/// let result = exponential_mechanism(eta, &outcomes, util_fn,
///                                     utility_min, utility_max,
///                                     max_outcomes,
///                                     rng, options);
/// ```
///
/// ## Exact Arithmetic
/// This function calls `enter_exact_scope()` and
/// `exit_exact_scope()`, and therefore clears the `mpfr::flags` and **does not preserve the
/// incoming flag state.**
pub fn exponential_mechanism<'a, T, R: ThreadRandGen, F: Fn(&T) -> f64>(
    eta: Eta,
    outcomes: &'a Vec<T>,
    utility: F,
    utility_min: u32,
    utility_max: u32,
    max_outcomes: u32,
    rng: &mut R,
    options: ExponentialOptions,
) -> Fallible<&'a T> {
    // Check Parameters
    eta.check()?;
    if (max_outcomes as usize) < outcomes.len() {
        return fallible!(FailedFunction, "Number of outcomes exceeds max_outcomes.");
    }

    // Generate an ExponentialConfig
    let mut exponential_config = ExponentialConfig::new(
        eta,
        utility_min,
        utility_max,
        max_outcomes,
        options.empirical_precision,
        options.min_retries,
    )?;

    // Compute Utilities
    let mut utilities = Vec::new();
    for o in outcomes.iter() {
        let mut u = utility(o);
        // clamp the utility to the allowed range
        if u > exponential_config.utility_max as f64 {
            u = exponential_config.utility_max as f64;
        } else if u < exponential_config.utility_min as f64 {
            u = exponential_config.utility_min as f64;
        }
        utilities.push(randomized_round(
            u,
            &mut exponential_config.arithmetic_config,
            rng,
        ));
    }

    // Enter exact scope
    exponential_config.arithmetic_config.enter_exact_scope()?;

    // get the base
    let base = &exponential_config.get_base();

    // Generate weights vector
    let mut weights = Vec::new();
    for u in utilities.iter() {
        let w = exponential_config.arithmetic_config.get_float(base.pow(u));
        weights.push(w);
    }

    // Sample
    let sample_index = normalized_sample(
        &weights,
        &mut exponential_config.arithmetic_config,
        rng,
        options.optimized_sample,
    )?;
    let sample = &outcomes[sample_index];

    // Exit exact scope
    exponential_config.arithmetic_config.exit_exact_scope()?;

    Ok(sample)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::samplers::GeneratorOpenDP;

    /// Runs the exponential mechanism multiple times
    #[test]
    fn test_exponential_mechanism_basic() {
        let num_samples = 1000;
        let num_outcomes = 5;
        let outcomes: Vec<u32> = (0..num_outcomes).collect();
        let eta = Eta::new(1, 1, 1).unwrap();
        let utility_min = 0;
        let utility_max = num_outcomes * 2;
        let max_outcomes = 10;
        let mut rng = GeneratorOpenDP::default();

        fn util_fn(x: &u32) -> f64 {
            return (*x as f64) * 2.0;
        }

        let options: ExponentialOptions = Default::default();
        let _outcome = exponential_mechanism(
            eta,
            &outcomes,
            util_fn,
            utility_min,
            utility_max,
            max_outcomes,
            &mut rng,
            options,
        );

        let mut samples = [0; 5];
        for _i in 0..num_samples {
            let sample = exponential_mechanism(
                eta,
                &outcomes,
                util_fn,
                utility_min,
                utility_max,
                max_outcomes,
                &mut rng,
                options,
            )
            .unwrap();
            samples[*sample as usize] += 1;
        }
    }
}
