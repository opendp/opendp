//! Implements the base-2 sparse vector mechanism.

use crate::error::Fallible;

use crate::measurements::b2dp::utilities::discretesampling::{
    adjust_eta, conditional_lazy_threshold, is_multiple_of, lazy_threshold, sample_within_bounds,
};
use crate::measurements::b2dp::utilities::exactarithmetic::ArithmeticConfig;
use crate::measurements::b2dp::utilities::params::Eta;
use rug::{rand::ThreadRandGen, Float};

/// The sparse vector mechanism
/// Sensitivity (Delta) is assumed to be 1. Privacy parameters must be scaled
/// appropriately if this is not the case.
///
/// ## Arguments:
///   * `eta1`: the privacy budget to spend on sampling the threshold
///   * `eta2`: the privacy budget to spend on the queries. Sensitivity of
///         queries assumed to be 1. `eta2` must be scaled appropriately for c.
///   * `c`: the number of allowed positive queries.
///   * `gamma`: the granularity (must be a reciprocal of a positive integer)
///   * `Q`: a set of query values which must be integer multiples of `gamma`, otherwise
///          they are rounded, which must be accounted for in privacy budget calculations.
///   * `Qmin`: the minimum value any query may be.
///   * `Qmax`: the maximum value any query may be.
///   * `w`: the width to use for threshold comparison (higher width increases
///         accuracy at the cost of efficiency, but only up to a point)
///   * `rng`: randomness source
///   * `optimize`: whether to optimize sampling (exacerbates timing channels)
///
/// ## Privacy budget usage
/// Uses `eta1*Delta + 2*eta2*Delta*c`. As such, `eta` and `eta2` should be
/// scaled appropriately for the desired usage.
///
/// ## Returns
/// A vector of `bool` with at most `c` `true` entries indicating which queries
/// exceeded the noisy threshold or an error if insufficient precision to do
/// so.
///
/// ## Exact Arithmetic
/// This method calls `enter_exact_scope` which clears the `mpfr::flags`.
///
/// ## Timing Channels
/// * This method uses [normalized_sample](../../utilities/exactarithmetic/fn.normalized_sample.html#known-timing-channels)
/// which has known timing channels.
///  * This method uses [`get_sum`](../../utilities/discretesampling/fn.get_sum.html),
///   which has a known timing channel.
/// Care should be taken given that this mechanism allows for an arbitrary
/// number of queries which may amplify timing channels.
pub fn sparse_vector<R: ThreadRandGen>(
    eta1: Eta,
    eta2: Eta,
    c: usize,
    query_values: &Vec<f64>,
    gamma: f64,
    query_min: f64,
    query_max: f64,
    w: f64,
    rng: &mut R,
    optimize: bool,
) -> Fallible<Vec<bool>> {
    // Initialize output vector
    let mut outputs: Vec<bool> = Vec::new();
    let mut count = 0;
    // Get an arithmetic_config to keep track of required precision
    let mut arithmetic_config = ArithmeticConfig::basic()?;
    let mut g;
    let mut g_inv;

    // Loop until test computations complete successfully
    loop {
        arithmetic_config.enter_exact_scope()?;
        // Check gamma and adjustments to ensure sufficient precision
        // `adjust_eta` also checks for validity of gamma and that
        // g_inv is integer.
        g = arithmetic_config.get_float(gamma);
        g_inv = arithmetic_config.get_float(1.0 / gamma);
        let _eta1_prime = adjust_eta(eta1, &g_inv, &mut arithmetic_config)?;
        let _eta2_prime = adjust_eta(eta2, &g_inv, &mut arithmetic_config)?;
        // Try to sample rho
        let _rho = sample_within_bounds(
            eta1,
            &arithmetic_config.get_float(&g),
            &arithmetic_config.get_float(query_min - w),
            &arithmetic_config.get_float(query_max + w),
            &mut arithmetic_config,
            rng,
            optimize,
        );

        // Compute maximum and minimum possible calls to lazy_threshold()
        // which will result from `query_min - w` and
        // `query_max + w`
        let mut thresh = arithmetic_config.get_float(query_min - w);
        let mut _test =
            lazy_threshold(eta2, &mut arithmetic_config, &g_inv, &thresh, rng, optimize);

        thresh = arithmetic_config.get_float(query_max + w);
        _test = lazy_threshold(eta2, &mut arithmetic_config, &g_inv, &thresh, rng, optimize);
        let ex = arithmetic_config.exit_exact_scope();
        // If computations suceeded, exit the loop
        if ex.is_ok() {
            break;
        }
        // Otherwise, increase precision and try again
        // `increase_precision` gives an error if max system
        // precision is exceeded.
        arithmetic_config.increase_precision(16)?;
        // reset the inexact arithmetic flag on the configuration
        arithmetic_config.inexact_arithmetic = false;
    }
    // re-enter the exact scope after appropriate precision determined
    arithmetic_config.enter_exact_scope()?;

    // Check that w, Qmax, Qmin, etc are multiples of gamma
    if !is_multiple_of(&arithmetic_config.get_float(w), &g) {
        return fallible!(FailedFunction, "w is not an integer multiple of gamma.");
    }
    if !is_multiple_of(&arithmetic_config.get_float(query_min), &g) {
        return fallible!(
            FailedFunction,
            "query_min is not an integer multiple of gamma."
        );
    }
    if !is_multiple_of(&arithmetic_config.get_float(query_max), &g) {
        return fallible!(
            FailedFunction,
            "query_max is not an integer multiple of gamma."
        );
    }

    // Sample Rho
    let rho = sample_within_bounds(
        eta1,
        &arithmetic_config.get_float(&g),
        &arithmetic_config.get_float(query_min - w),
        &arithmetic_config.get_float(query_max + w),
        &mut arithmetic_config,
        rng,
        optimize,
    )?;

    // Iterate through queries
    for i in 0..query_values.len() {
        // Clamp Q[i] if needed
        let mut q: Float;
        if query_values[i] > query_max {
            q = arithmetic_config.get_float(query_max);
        } else if query_values[i] < query_min {
            q = arithmetic_config.get_float(query_min);
        } else {
            q = arithmetic_config.get_float(query_values[i]);
        }

        // Check that q is a multiple of gamma
        if !is_multiple_of(&q, &g) {
            // Round
            let mut m = arithmetic_config.get_float(&q / &g);
            m.round_mut();
            q = arithmetic_config.get_float(&g * &m);
        }

        // Compute rho hat
        let rho_hat: Float;
        let rho_max = arithmetic_config.get_float(&q + w);
        let rho_min = arithmetic_config.get_float(&q - w);
        if rho > rho_max {
            rho_hat = rho_max;
        } else if rho < rho_min {
            rho_hat = rho_min;
        } else {
            rho_hat = arithmetic_config.get_float(&rho);
        }

        // Run noisy threshold
        let g_inv = arithmetic_config.get_float(1.0 / gamma);
        // Noisy threshold for `rho - query_values[i]`
        let thresh = arithmetic_config.get_float(&rho_hat - &q);

        let a = lazy_threshold(eta2, &mut arithmetic_config, &g_inv, &thresh, rng, optimize)?;
        if a.is_infinite() && a.is_sign_positive() {
            outputs.push(true);
            count += 1;
            // if we have already encountered c positives, stop
            if count >= c {
                return Ok(outputs);
            }
        }
        // Otherwise, output false
        else {
            outputs.push(false);
        }
    }
    // Exit the exact scope and check if inexact arithmetic
    arithmetic_config.exit_exact_scope()?;

    return Ok(outputs);
}

/// The sparse vector mechanism (with gap information)
/// Sensitivity (Delta) is assumed to be 1. Privacy parameters must be scaled
/// appropriately if this is not the case.
///
/// ## Arguments:
///   * `eta1`: the privacy budget to spend on sampling the threshold
///   * `eta2`: the privacy budget to spend on the queries. Sensitivity of
///         queries assumed to be 1. `eta2` must be scaled appropriately for c.
///   * `c`: the number of allowed positive queries.
///   * `gaps`: a set of gap values greater than zero (must be integer multiples of gamma).
///   * `gamma`: the granularity (must be a reciprocal of a positive integer)
///   * `Q`: a set of query values which must be integer multiples of `gamma`, otherwise
///          they are rounded, which must be accounted for in privacy budget calculations.
///   * `Qmin`: the minimum value any query may be.
///   * `Qmax`: the maximum value any query may be.
///   * `w`: the width to use for threshold comparison (higher width increases
///         accuracy at the cost of efficiency, but only up to a point)
///   * `rng`: randomness source
///   * `optimize`: whether to optimize sampling (exacerbates timing channels)
///
/// ## Privacy budget usage
/// Uses `eta1*Delta + 2*eta2*Delta*c`. As such, `eta` and `eta2` should be
/// scaled appropriately for the desired usage.
///
/// ## Returns
/// A vector of `Option<f64>` with at most `c` `Some` entries indicating which
/// queries exceeded the noisy threshold and the maximum gap exceeded or an
/// error if insufficient precision to do so.
///
/// ## Exact Arithmetic
/// This method calls `enter_exact_scope` which clears the `mpfr::flags`.
///
/// ## Timing Channels
/// * This method uses [normalized_sample](../../utilities/exactarithmetic/fn.normalized_sample.html#known-timing-channels)
/// which has known timing channels.
///  * This method uses [`get_sum`](../../utilities/discretesampling/fn.get_sum.html),
///   which has a known timing channel.
/// Care should be taken given that this mechanism allows for an arbitrary
/// number of queries which may amplify timing channels.
pub fn sparse_vector_with_gap<R: ThreadRandGen>(
    eta1: Eta,
    eta2: Eta,
    c: usize,
    gaps: &Vec<f64>,
    query_values: &Vec<f64>,
    gamma: f64,
    query_min: f64,
    query_max: f64,
    w: f64,
    rng: &mut R,
    optimize: bool,
) -> Fallible<Vec<Option<f64>>> {
    // Initialize output vector
    let mut outputs: Vec<Option<f64>> = Vec::new();
    let mut count = 0;

    // Get an arithmetic_config to keep track of required precision
    let mut arithmetic_config = ArithmeticConfig::basic()?;
    let mut g;
    let mut g_inv;

    // Loop until test computations complete successfully
    loop {
        arithmetic_config.enter_exact_scope()?;
        // Check gamma and adjustments to ensure sufficient precision
        // `adjust_eta` also checks for validity of gamma and that
        // g_inv is integer.
        g = arithmetic_config.get_float(gamma);
        g_inv = arithmetic_config.get_float(1.0 / gamma);
        let _eta1_prime = adjust_eta(eta1, &g_inv, &mut arithmetic_config)?;
        let _eta2_prime = adjust_eta(eta2, &g_inv, &mut arithmetic_config)?;

        // Try to sample rho
        let _rho = sample_within_bounds(
            eta1,
            &arithmetic_config.get_float(&g),
            &arithmetic_config.get_float(query_min - w),
            &arithmetic_config.get_float(query_max + w),
            &mut arithmetic_config,
            rng,
            optimize,
        );

        // Compute maximum and minimum possible calls to lazy_threshold()
        // which will result from `query_min - w` and
        // `query_max + w`
        let mut thresh = arithmetic_config.get_float(query_min - w);
        let mut _test =
            lazy_threshold(eta2, &mut arithmetic_config, &g_inv, &thresh, rng, optimize);

        thresh = arithmetic_config.get_float(query_max + w);
        _test = lazy_threshold(eta2, &mut arithmetic_config, &g_inv, &thresh, rng, optimize);
        let ex = arithmetic_config.exit_exact_scope();
        // If computations suceeded, exit the loop
        if ex.is_ok() {
            break;
        }
        // Otherwise, increase precision and try again
        // `increase_precision` gives an error if max system
        // precision is exceeded.
        arithmetic_config.increase_precision(16)?;
        // reset the inexact arithmetic flag on the configuration
        arithmetic_config.inexact_arithmetic = false;
    }

    // re-enter the exact scope after appropriate precision determined
    arithmetic_config.enter_exact_scope()?;

    // Check that w, Qmax, Qmin, etc are multiples of gamma
    if !is_multiple_of(&arithmetic_config.get_float(w), &g) {
        return fallible!(FailedFunction, "w is not an integer multiple of gamma.");
    }
    if !is_multiple_of(&arithmetic_config.get_float(query_min), &g) {
        return fallible!(
            FailedFunction,
            "query_min is not an integer multiple of gamma."
        );
    }
    if !is_multiple_of(&arithmetic_config.get_float(query_max), &g) {
        return fallible!(
            FailedFunction,
            "query_max is not an integer multiple of gamma."
        );
    }

    // Check that all gaps are non-negative integer multiples of gamma
    for i in 0..gaps.len() {
        if !is_multiple_of(&arithmetic_config.get_float(gaps[i]), &g) {
            return fallible!(
                FailedFunction,
                "All gaps must be integer multiples of gamma."
            );
        }
        if gaps[i] < 0.0 {
            return fallible!(FailedFunction, "All gaps must be non-negative.");
        }
    }

    // Sample Rho
    let rho = sample_within_bounds(
        eta1,
        &arithmetic_config.get_float(&g),
        &arithmetic_config.get_float(query_min - w),
        &arithmetic_config.get_float(query_max + w),
        &mut arithmetic_config,
        rng,
        optimize,
    )?;

    // Iterate through queries
    for i in 0..query_values.len() {
        // Clamp Q[i] if needed
        let mut q: Float;
        if query_values[i] > query_max {
            q = arithmetic_config.get_float(query_max);
        } else if query_values[i] < query_min {
            q = arithmetic_config.get_float(query_min);
        } else {
            q = arithmetic_config.get_float(query_values[i]);
        }

        // Check that q is a multiple of gamma
        if !is_multiple_of(&q, &g) {
            // Round
            let mut m = arithmetic_config.get_float(&q / &g);
            m.round_mut();
            q = arithmetic_config.get_float(&g * &m);
        }

        // Compute rho hat
        let rho_hat: Float;
        let rho_max = arithmetic_config.get_float(&q + w);
        let rho_min = arithmetic_config.get_float(&q - w);
        if rho > rho_max {
            rho_hat = rho_max;
        } else if rho < rho_min {
            rho_hat = rho_min;
        } else {
            rho_hat = arithmetic_config.get_float(&rho);
        }

        // Run noisy threshold
        let g_inv = arithmetic_config.get_float(1.0 / gamma);
        // Noisy threshold for `rho - query_values[i]`
        let thresh = arithmetic_config.get_float(&rho_hat - &q);

        let a = lazy_threshold(eta2, &mut arithmetic_config, &g_inv, &thresh, rng, optimize)?;
        if a.is_infinite() && a.is_sign_positive() {
            let mut max_gap = 0.0;
            // the conditional threshold
            let mut conditional_thresh = arithmetic_config.get_float(&thresh);
            // Check conditional thresholds
            for i in 0..gaps.len() {
                // The next threshold to pass
                let thresh = arithmetic_config.get_float(&thresh + gaps[i]);
                let ai = conditional_lazy_threshold(
                    eta2,
                    &mut arithmetic_config,
                    &g_inv,
                    &thresh,
                    &conditional_thresh,
                    rng,
                    optimize,
                )?;
                if ai.is_infinite() && ai.is_sign_positive() {
                    // update the maximum gap and conditional threshold
                    max_gap = gaps[i];
                    conditional_thresh = arithmetic_config.get_float(&thresh);
                } else {
                    // quit with the current gap
                    break;
                }
            }

            outputs.push(Some(max_gap));
            count += 1;
            // if we have already encountered c positives, stop
            if count >= c {
                return Ok(outputs);
            }
        }
        // Otherwise, output false
        else {
            outputs.push(None);
        }
    }
    // Exit the exact scope and check if inexact arithmetic
    arithmetic_config.exit_exact_scope()?;

    return Ok(outputs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::samplers::GeneratorOpenDP;

    #[test]
    fn test_sparse_vector_with_gap() {
        let eta1 = Eta::new(1, 1, 2).unwrap();
        let eta2 = Eta::new(1, 1, 2).unwrap();
        let c = 2;
        let queries = vec![
            -10.0, // Should be clamped based on q_min
            -1.0, 5.0, // Very likely to be positive
            -5.0, 0.0, -2.0, 1.0, -2.0, -3.0, -4.0, 5.0, 1.0,
        ];
        let gaps = vec![1.0, 2.0, 3.0, 4.0];
        let gamma = 0.5;
        let q_min = -5.0;
        let q_max = 6.0;
        let w = 10.0;
        let mut rng = GeneratorOpenDP::default();
        let optimize = false;
        let outputs = sparse_vector_with_gap(
            eta1, eta2, c, &gaps, &queries, gamma, q_min, q_max, w, &mut rng, optimize,
        );
        println!("{:?}", outputs);
        assert!(outputs.is_ok());
        let outputs = outputs.unwrap();
        assert!(outputs.len() >= c); // Should always pass.
        println!("\n{:?}", outputs);
    }

    #[test]
    fn test_sparse_vector() {
        let eta1 = Eta::new(1, 1, 2).unwrap();
        let eta2 = Eta::new(1, 1, 2).unwrap();
        let c = 2;
        let queries = vec![
            -10.0, // Should be clamped based on q_min
            -1.0, 5.0, // Very likely to be positive
            -5.0, 0.0, -2.0, 1.0, -2.0, -3.0, -4.0, 5.0, 1.0,
        ];
        let gamma = 0.5;
        let q_min = -5.0;
        let q_max = 6.0;
        let w = 10.0;
        let mut rng = GeneratorOpenDP::default();
        let optimize = false;
        let outputs = sparse_vector(
            eta1, eta2, c, &queries, gamma, q_min, q_max, w, &mut rng, optimize,
        );
        assert!(outputs.is_ok());
        let outputs = outputs.unwrap();
        assert!(outputs.len() >= c); // Should always pass.
        println!("\n{:?}", outputs);
    }

    #[test]
    fn test_few_positives_sparse_vector() {
        let eta1 = Eta::new(1, 1, 2).unwrap();
        let eta2 = Eta::new(1, 1, 2).unwrap();
        let c = 3;
        // All queries are true negatives, very few should
        // be positive. Tests that zero or too few positives
        // behavior is correct.
        let queries = vec![
            -10.0, // Should be clamped based on q_min
            -4.0, -5.0, -5.0, -2.0, -4.0, -2.0, -3.0, -4.0, -5.0, -1.0,
        ];
        let gamma = 0.5;
        let q_min = -5.0;
        let q_max = 6.0;
        let w = 10.0;
        let mut rng = GeneratorOpenDP::default();
        let optimize = false;
        let outputs = sparse_vector(
            eta1, eta2, c, &queries, gamma, q_min, q_max, w, &mut rng, optimize,
        );
        assert!(outputs.is_ok());
        let outputs = outputs.unwrap();
        assert!(outputs.len() >= c); // Should always pass.
        assert!(outputs.len() > 6); // May fail with very small probability
        println!("\n{:?}", &outputs);
    }

    #[test]
    fn test_sparse_vector_precision() {
        let eta1 = Eta::new(1, 1, 2).unwrap();
        let eta2 = Eta::new(1, 1, 2).unwrap();
        let c = 2;
        let queries = vec![
            -10.0, // Should be clamped based on q_min
            -1.0, 5.0, // Very likely to be positive
            -5.0, 0.0, -2.0, 1.0, -2.0, -3.0, -4.0, 5.0, 1.0,
        ];
        let gamma = 0.5;
        let q_min = -50.0; // Larger range of q_min, q_max and
        let q_max = 60.0; // w will require greater precision
        let w = 100.0; // than default.
        let mut rng = GeneratorOpenDP::default();
        let optimize = false;
        let outputs = sparse_vector(
            eta1, eta2, c, &queries, gamma, q_min, q_max, w, &mut rng, optimize,
        );
        assert!(outputs.is_ok());
        let outputs = outputs.unwrap();
        assert!(outputs.len() >= c); // Should always pass.
        println!("\n{:?}", outputs);
    }
}
