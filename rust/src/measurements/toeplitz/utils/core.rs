use crate::error::*;
use crate::traits::Integer;
use dashu::integer::IBig;
use num::{CheckedAdd, CheckedMul, CheckedSub};
use std::fmt::Display;
use std::str::FromStr;

use super::noise_generation::{compute_correlated_noise_at_time, apply_correlated_noise};
use super::type_conversion::compute_prefix_sum;

/// CORE ALGORITHM (shared by both one-shot and continual release): Apply Toeplitz mechanism to a range of time indices
/// 
/// This is the stateless utility function that computes noisy prefix sums
/// for times [start_time, end_time) given pre-existing noise.
/// 
/// # Mathematical Foundation
/// For each time t, this computes:
/// 1. s_t = Σ_{i=0}^t g_i (the true prefix sum)
/// 2. \tilde{z}_t = Σ_{i=0}^t c'_{t-i} * Z[i] (correlated noise via inverse Toeplitz)
/// 3. \tilde{s}_t = s_t + \tilde{z}_t (noisy prefix sum)
/// 
/// This function is kept private and is exposed in two ways:
/// 1. For a one-time, end-to-end computation, call `compute_toeplitz_mechanism`.
/// 2. For continual release, call `release` associated with the corresponding `ContinualToeplitz` object to update the noise history and compute new noisy prefix sums.
pub(crate) fn compute_toeplitz_range<T>(
    data: &[T],
    raw_noise: &[IBig],
    start_time: usize,
    end_time: usize,
    scale_bits: usize,
) -> Fallible<Vec<T>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd,
{
    // Validate inputs
    if end_time > data.len() {
        return fallible!(FailedFunction, "end_time {} exceeds data length {}", end_time, data.len());
    }
    if end_time > raw_noise.len() {
        return fallible!(FailedFunction, "end_time {} exceeds noise length {}", end_time, raw_noise.len());
    }
    
    let mut output = Vec::with_capacity(end_time - start_time);
    
    for t in start_time..end_time {
        // Step 1: Compute the t-th prefix sum (unweighted sum from 0 to t)
        let prefix_sum = compute_prefix_sum(&data, t)?;
        
        // Step 2: Compute correlated noise using inverse Toeplitz transformation
        let correlated_noise = compute_correlated_noise_at_time(
            t, 
            raw_noise, 
            scale_bits
        )?;
        
        // Step 3: Apply noise and convert to output type
        let noisy_sum = apply_correlated_noise::<T>(
            prefix_sum,
            correlated_noise,
            scale_bits
        )?;
        
        output.push(noisy_sum);
    }
    
    Ok(output)
}
