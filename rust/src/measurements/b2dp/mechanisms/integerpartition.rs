//! Implements the integer partition exponential mechanism (fixed bounds variant).

use crate::error::Fallible;
use crate::measurements::b2dp::utilities::{
    bounds::PartitionBound, weights::Key, weights::WeightTable,
};
use crate::measurements::b2dp::{normalized_sample, Eta};
use crate::traits::samplers::GeneratorOpenDP;
use rug::Float;
use std::cmp;

#[derive(Debug, Clone, Copy)]
pub struct IntegerPartitionOptions {
    /// Whether to optimize sampling, range computation, etc.
    /// Exacerbates timing channels
    pub optimize: bool,
}

impl Default for IntegerPartitionOptions {
    /// Default options for the exponential mechanism
    /// `min_retries = 1`, `optimized_sample = false`, `empirical_precision = false`
    fn default() -> IntegerPartitionOptions {
        IntegerPartitionOptions { optimize: false }
    }
}

/// The integer partition mechanism given a pre-computed weight table.
///
/// ## Arguments
///   * `weight_table`: a pre-computed weight table for the integer partition
///     exponential mechanism.
///   * `partition_bounds`:
///
/// ## Returns
/// Returns a vector representing a private approximation of the target integer
/// partition or an error if the partition could not be approximated.
///
/// ## Privacy Budget
/// (Should be accounted for in weight computation.)
///
/// ## Exact Arithmetic
/// This function calls `enter_exact_scope()` and
/// `exit_exact_scope()`, and therefore clears the `mpfr::flags` and **does not preserve the
/// incoming flag state.** This method may need to be re-tried with higher precision on failure.
///
/// ## Timing Channels
/// **This mechanism has known timing channels.** Please see
/// [normalized_sample](../../utilities/exactarithmetic/fn.normalized_sample.html#known-timing-channels).
pub fn integer_partition_mechanism_with_weights(
    weight_table: &mut WeightTable,
    pb: &PartitionBound,
    options: IntegerPartitionOptions,
) -> Fallible<Vec<i64>> {
    // Check parameters
    pb.check()?;

    // Initialize parameters
    let n = pb.upper.len();
    let mut y_prev = pb.upper[0]; // Initialize the maximum value for the first
                                  // index of the output vector y.
    let mut y: Vec<i64> = Vec::new(); // the vector storing outcomes.

    // enter the exact arithmetic scope
    weight_table.arithmetic_config.enter_exact_scope()?;

    // Sampling loop
    for i in 0..n {
        let q_max = cmp::min(y_prev, pb.upper[i]);
        // Get the set of weights for this index
        let mut weight_list: Vec<Float> = Vec::new();
        for q in pb.lower[i]..(q_max + 1) {
            // inclusive
            let k = Key { i, q };
            if let Some(wt) = weight_table.weights.get(&k) {
                let w = Float::with_val(weight_table.arithmetic_config.precision, wt);
                weight_list.push(w);
            } else {
                return fallible!(FailedFunction, "Weight table missing value.");
            }
        }
        // Sample the next value
        let mut rng = GeneratorOpenDP::default();
        let sample = normalized_sample(
            &weight_list,
            &mut weight_table.arithmetic_config,
            &mut rng,
            options.optimize,
        )?;
        let y_i = pb.lower[i] + sample as i64;
        if y_i > q_max {
            return fallible!(FailedFunction, "Bad sample.");
        }

        // Add the next index to the output and update y_prev
        y.push(y_i);
        y_prev = y_i;
    }
    // exit the exact arithmetic scope
    weight_table.arithmetic_config.exit_exact_scope()?;

    return Ok(y);
}

/// The integer partition mechanism given bounds
///
/// ## Arguments
///   * `eta`: privacy parameter
///   * `x`: the target (private) integer partition
///   * `pb`: the partition bounds
///   * `options`: mechanism options
///
/// ## Returns
/// Returns a vector representing a private approximation of the target integer
/// partition or an error if the partition could not be approximated.
///
/// ## Privacy Budget
/// Consumes `2*eta` in privacy budget for single add/remove adjacency.
///
/// ## Exact Arithmetic
/// This function calls `enter_exact_scope()` and
/// `exit_exact_scope()`, and therefore clears the `mpfr::flags` and **does not preserve the
/// incoming flag state.** This method may need to be re-tried (or `integer_partition_mechanism_with_weights`
/// may be used) with higher precision on failure.
///
/// ## Timing Channels
/// This method increases precision as needed dependign on private data,
/// and should be treated as unsafe for timing channels.
pub fn integer_partition_mechanism_with_bounds(
    eta: Eta,
    x: &Vec<i64>,
    pb: &PartitionBound,
    options: IntegerPartitionOptions,
) -> Fallible<Vec<i64>> {
    // Check parameters
    pb.check()?;
    eta.check()?;

    // Initialize parameters
    let n = pb.upper.len();
    let mut y_prev = pb.upper[0]; // Initialize the maximum value for the first
                                  // index of the output vector y.
    let mut y: Vec<i64> = Vec::new(); // the vector storing outcomes.

    // Compute the weights
    let mut weight_table = WeightTable::from_bounds(eta, pb, x)?;

    let inc = weight_table.arithmetic_config.precision;
    weight_table.arithmetic_config.increase_precision(inc)?;
    // enter the exact arithmetic scope
    weight_table.arithmetic_config.enter_exact_scope()?;

    // Sampling loop
    for i in 0..n {
        let q_max = cmp::min(y_prev, pb.upper[i]);
        // Get the set of weights for this index
        let mut weight_list: Vec<Float> = Vec::new();
        for q in pb.lower[i]..(q_max + 1) {
            // inclusive
            let k = Key { i, q };
            if let Some(wt) = weight_table.weights.get(&k) {
                let w = Float::with_val(weight_table.arithmetic_config.precision, wt);
                weight_list.push(w);
            } else {
                return fallible!(FailedFunction, "Weight table missing value.");
            }
        }

        // Sample the next value
        let mut rng = GeneratorOpenDP::default();
        let sample = normalized_sample(
            &weight_list,
            &mut weight_table.arithmetic_config,
            &mut rng,
            options.optimize,
        )?;
        // // exit the exact arithmetic scope
        // weight_table.arithmetic_config.exit_exact_scope()?;
        // // enter the exact arithmetic scope
        // weight_table.arithmetic_config.enter_exact_scope()?;
        // println!("{:?}: {:?}", i, weight_table.arithmetic_config);
        let y_i = pb.lower[i] + sample as i64;
        if y_i > q_max {
            return fallible!(FailedFunction, "Bad sample.");
        }

        // Add the next index to the output and update y_prev
        y.push(y_i);
        y_prev = y_i;
    }
    // exit the exact arithmetic scope
    weight_table.arithmetic_config.exit_exact_scope()?;

    return Ok(y);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_with_weights() {
        let eta = Eta::new(1, 1, 1).unwrap();
        let f: Vec<i64> = vec![5, 4, 3, 2, 1];
        let n: i64 = f.iter().sum();
        let pb = PartitionBound::new(n as usize).unwrap();
        let options: IntegerPartitionOptions = Default::default();
        let mut weight_table = WeightTable::from_bounds(eta, &pb, &f).unwrap();
        let result = integer_partition_mechanism_with_weights(&mut weight_table, &pb, options);
        assert!(result.is_ok());
        let y = result.unwrap();
        // check that output is in bounds
        for i in 0..y.len() {
            assert!(y[i] <= pb.upper[i]);
            assert!(y[i] >= pb.lower[i]);
        }
    }

    #[test]
    fn test_small_partition() {
        let eta = Eta::new(1, 1, 1).unwrap();
        let f: Vec<i64> = vec![5, 4, 3, 2, 1];
        let n: i64 = f.iter().sum();
        let pb = PartitionBound::new(n as usize).unwrap();
        let options: IntegerPartitionOptions = Default::default();
        let result = integer_partition_mechanism_with_bounds(eta, &f, &pb, options);
        assert!(result.is_ok());
        let y = result.unwrap();
        // check that output is in bounds
        for i in 0..y.len() {
            assert!(y[i] <= pb.upper[i]);
            assert!(y[i] >= pb.lower[i]);
        }
    }

    #[test]
    fn test_medium_partition() {
        let eta = Eta::new(1, 1, 1).unwrap();
        let n: i64 = 200;
        let mut x: Vec<i64> = (0..n).map(|x| x).rev().collect();

        for i in (n as usize / 2)..(n as usize) {
            x[i] = 0;
        }

        let total_count: i64 = x.iter().sum();

        let pb = PartitionBound::with_cells(total_count as usize, n as usize).unwrap();

        let options: IntegerPartitionOptions = Default::default();
        let result = integer_partition_mechanism_with_bounds(eta, &x, &pb, options);
        println!("{:?}", result);
        assert!(result.is_ok());

        let y = result.unwrap();

        // Check that y is proper length
        assert_eq!(y.len(), pb.upper.len());

        // check that output is in bounds
        for i in 0..x.len() {
            assert!(y[i] <= pb.upper[i]);
            assert!(y[i] >= pb.lower[i]);
            //println!("{} <= {}:{} <= {}",pb.lower[i], y[i], x[i], pb.upper[i]);
        }
    }
}
