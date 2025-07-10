use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::measures::MaxDivergence;
use crate::metrics::L1Distance;
use crate::traits::{Integer, InfCast};

use crate::traits::samplers::sample_discrete_gaussian;
use dashu::rational::RBig;
use dashu::integer::IBig;

use num::{Zero, CheckedAdd, CheckedMul, CheckedSub};
use std::fmt::Display;
use std::str::FromStr;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[cfg(feature = "contrib-continual")]
mod continual;

#[cfg(feature = "contrib-continual")]
pub use continual::ContinualToeplitz;


/// Make a measurement that adds correlated noise for continual release of counting statistics
/// using the Toeplitz mechanism.
/// 
/// This implements the basic Toeplitz mechanism from Section 2.3 of https://arxiv.org/abs/2506.08201, 
/// named "Correlated Noise Mechanisms for Differentially Private Learning" by Pillutla et al.,
/// which achieves near-optimal utility for releasing prefix sums with differential privacy.
/// Refer to chapter 2 of the linked survey for more theoretical results about continual release with DP.
/// 
/// The mechanism adds correlated discrete Gaussian noise to enable accurate range queries
/// over streaming count data. A 2023 paper by Fichtenberger et al first came up with this basic 
/// Toeplitz-based construction, with the name of "Constant matters: Fine-grained Complexity of
/// Differentially Private Continual Observation", accessible at https://arxiv.org/abs/2202.11205.
/// 
/// # Arguments
/// * `input_domain` - VectorDomain with known size containing the time series of counts
/// * `input_metric` - L1Distance metric for measuring sensitivity  
/// * `scale` - Noise scale parameter ($\sigma$ as in the paper)
/// 
/// # Returns
/// A Measurement that computes differentially private prefix sums of counts.
/// 
/// # Example
/// ```
/// # use opendp::prelude::*;
/// # use opendp::measurements::make_toeplitz;
/// let input_domain = VectorDomain::new(
///     AtomDomain::<i32>::default()
/// ).with_size(10);
/// let input_metric = L1Distance::<i32>::default();
/// let scale = 2.0;
/// 
/// let measurement = make_toeplitz(input_domain, input_metric, scale)?;
/// let counts = vec![5i32; 10]; // 10 time steps with 5 counts each
/// let noisy_prefix_sums = measurement.invoke(&counts)?;
/// # Ok::<(), opendp::error::Fallible>(())
/// ```
pub fn make_toeplitz<T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L1Distance<T>,
    scale: f64,
) -> Fallible<Measurement<
    VectorDomain<AtomDomain<T>>,
    Vec<T>,
    L1Distance<T>,
    MaxDivergence,
>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd,
    f64: InfCast<T>,
{
    // Validate scale parameter
    if scale.is_sign_negative() || scale.is_zero() || !scale.is_finite() {
        return fallible!(MakeMeasurement, "scale must be positive and finite");
    }
    
    // Check that input domain has known size
    let n = match input_domain.size {
        Some(size) => size,
        None => return fallible!(MakeMeasurement, "input domain must have known size for Toeplitz mechanism"),
    };
    
    if n == 0 {
        return fallible!(MakeMeasurement, "input domain size must be positive");
    }
    
    let scale_bits = 40usize;  // Precision for fixed-point arithmetic
    
    // Create the measurement
    Measurement::new(
        input_domain.clone(),
        Function::new_fallible(move |data: &Vec<T>| -> Fallible<Vec<T>> {
            // Validate data length
            if data.len() != n {
                return fallible!(FailedFunction, "data length {} does not match domain size {}", data.len(), n);
            }
            
            // Apply the Toeplitz mechanism
            let mut noisy_sums = compute_toeplitz_mechanism(data, scale, scale_bits)?;
            
            // Clamp to non-negative before isotonic regression
            for i in 0..noisy_sums.len() {
                if noisy_sums[i] < T::zero() {
                    noisy_sums[i] = T::zero();
                }
            }
            
            // Apply isotonic regression to ensure monotonicity
            apply_isotonic_regression(noisy_sums)
        }),
        input_metric.clone(),
        MaxDivergence,
        PrivacyMap::new_fallible(move |d_in: &T| {
            // For pure DP with Gaussian noise: ε = (Δ * d_in) / σ
            // Δ = 1 for an individual is assumed, since:
            // - We don't assume the ability to be able to identify all counts that belong to a single user.
            // - We assume the simple use cases where an individual particilates 
            let d_in_f64 = f64::inf_cast(d_in.clone())?;
            Ok(d_in_f64 / scale)
        }),
    )
}


/// Implements Theorem 2.5 - Max-Loss-Optimal Toeplitz Factorization
/// For the unweighted prefix sum workload A = A_pre, the optimal factorization
/// is B_Toep = C_Toep = A_pre^(1/2)
mod toeplitz_coefficients {
    use super::*;
    
    /// Compute the optimal Toeplitz coefficient c*_t:
    /// c*_t = (-1)^t \cdot {-1/2 \choose t} = 2^{-2t} \cdot {2t \choose t}
    /// 
    /// This is the t-th coefficient of the matrix C = A_pre^(1/2). Note that a Toeplitz matrix is one where you can describve the entire matrix by its first column.
    /// For numerical stability, we scale up by 2^{scale\_bits}.
    pub fn compute_optimal_coefficient_scaled(t: usize, scale_bits: usize) -> Fallible<IBig> {
        if t == 0 {
            // c*_0 = 1 (scaled by 2^scale_bits)
            return Ok(IBig::from(1) << scale_bits);
        }
        
        // From Remark 2.6: c*_t = 2^{-2t} \cdot {2t \choose t}
        // Compute {2t \choose t} = (2t)! / (t! \cdot t!)
        let mut numerator = IBig::from(1);
        let mut denominator = IBig::from(1);
        
        // Efficient computation: {2t \choose t} = (2t * (2t-1) * ... * (t+1)) / t!
        for i in 1..=t {
            numerator *= t + i;  // Multiply by (t+1), (t+2), ..., 2t
            denominator *= i;    // Multiply by 1, 2, ..., t
        }
        
        // The coefficient is: {2t \choose t} / 4^t * 2^{scale\_bits}
        // Since 4^t = 2^{2t}, this is: {2t \choose t} * 2^{scale\_bits - 2t}
        if 2 * t <= scale_bits {
            Ok((numerator << (scale_bits - 2 * t)) / denominator)
        } else {
            Ok(numerator / (denominator << (2 * t - scale_bits)))
        }
    }
    
    /// Compute the inverse Toeplitz coefficient c'_t for noise generation
    /// c'_t = (-1)^t * {1/2 \choose t}
    /// 
    /// From Remark 2.6: c'_t = c*_{t+1} - c*_t for t > 0
    /// This is the t-th coefficient of C^{-1}
    pub fn compute_inverse_coefficient_scaled(t: usize, scale_bits: usize) -> Fallible<IBig> {
        if t == 0 {
            // c'_0 = 1 (scaled by 2^{scale\_bits})
            return Ok(IBig::from(1) << scale_bits);
        }
        
        // Using the relation: c'_t = c*_{t+1} - c*_t
        let c_t = compute_optimal_coefficient_scaled(t, scale_bits)?;
        let c_t_plus_1 = compute_optimal_coefficient_scaled(t + 1, scale_bits)?;
        Ok(c_t_plus_1 - c_t)
    }
}

/// Section 2.3: Noise Generation Process
/// 
/// The core computation from Equation (2.3):
/// \tilde{g}_t = g_t + (C^(-1)Z)[t, :] = g_t + Σ_{τ=0}^t (C^(-1))[t, τ] Z[τ, :]
mod noise_generation {
    use super::*;
    use super::toeplitz_coefficients::*;
    
    /// Apply the inverse Toeplitz transformation to compute correlated noise
    /// This implements the summation in Equation (2.3) for a single time step
    pub fn compute_correlated_noise_at_time(
        t: usize,
        raw_noise: &[IBig],
        scale_bits: usize,
    ) -> Fallible<IBig> {
        let mut correlated_noise = IBig::zero();
        
        // Compute Σ_{i=0}^t c'_{t-i} * Z[i]
        // where c'_j are the inverse Toeplitz coefficients
        for i in 0..=t {
            let coeff = compute_inverse_coefficient_scaled(t - i, scale_bits)?;
            correlated_noise += &coeff * &raw_noise[i];
        }
        
        Ok(correlated_noise)
    }
    
    /// Scale down the correlated noise and add to data
    /// This completes the transformation: M(G) = B(CG + Z) = A(G + C^{-1}Z)
    pub fn apply_correlated_noise<T>(
        data_value: IBig,
        correlated_noise: IBig,
        scale_bits: usize,
    ) -> Fallible<T>
    where
        T: Integer + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One,
    {
        // Scale down the noise
        let scaled_noise = correlated_noise >> scale_bits;
        
        // Add noise to data
        let noisy_value = data_value + scaled_noise;
        
        // Convert to output type with saturation
        from_ibig_saturating::<T>(noisy_value)
    }
}

/// MAIN ALGORITHM: Compute the Toeplitz mechanism for prefix sums
/// 
/// This implements the Toeplitz mechanism.
/// The mechanism adds correlated discrete Gaussian noise to enable accurate
/// range queries over streaming count data.
pub fn compute_toeplitz_mechanism<T>(
    data: &[T],
    scale: f64,
    scale_bits: usize,
) -> Fallible<Vec<T>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd,
{
    let n = data.len();
    if n == 0 {
        return Ok(vec![]);
    }
    
    // Convert scale to variance for discrete Gaussian
    let variance = RBig::from((scale * scale * 1e9) as i64) / RBig::from(1_000_000_000i64);
    
    // Step 1: Generate independent discrete Gaussian noise Z
    let mut raw_noise = Vec::with_capacity(n);
    for _ in 0..n {
        raw_noise.push(sample_discrete_gaussian(variance.clone())?);
    }
    
    // Step 2: Apply Toeplitz mechanism to compute noisy prefix sums
    compute_toeplitz_range(data, &raw_noise, 0, n, scale_bits)
}

/// Apply Toeplitz mechanism to a range of time indices
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
fn compute_toeplitz_range<T>(
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
        let correlated_noise = noise_generation::compute_correlated_noise_at_time(
            t, 
            raw_noise, 
            scale_bits
        )?;
        
        // Step 3: Apply noise and convert to output type
        let noisy_sum = noise_generation::apply_correlated_noise::<T>(
            prefix_sum,
            correlated_noise,
            scale_bits
        )?;
        
        output.push(noisy_sum);
    }
    
    Ok(output)
}

/// Apply isotonic regression using the Pool Adjacent Violators Algorithm (PAVA)
/// 
/// PAVA runs in O(n) time and ensures that the output is the best MSE-fitting of the input data that respects non-decreasing monotonicity.
/// 
/// The post-processing property of differential privacy (Dwork et al., 2006) guarantees
/// that this deterministic transformation preserves the ε-differential privacy of the input.
/// Another way to think about this is: all the computations here can be done deterministically with the
/// noisy counts after the Toeplitz mechanism, through local computations by the adversary,
/// so the two views with or without this isotonic regression step are identical.
fn apply_isotonic_regression<T>(mut values: Vec<T>) -> Fallible<Vec<T>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd,
{
    if values.is_empty() {
        return Ok(values);
    }
    
    let n = values.len();
    let mut blocks = Vec::with_capacity(n);
    
    // Initialize each value as its own block (start_idx, end_idx, sum, count)
    for i in 0..n {
        blocks.push((i, i, to_ibig(&values[i])?, 1usize));
    }
    
    // Pool adjacent violators
    let mut i = 0;
    while i < blocks.len() - 1 {
        let (start1, end1, sum1, count1) = &blocks[i];
        let (start2, end2, sum2, count2) = &blocks[i + 1];
        
        // Check if monotonicity is violated (average of block i > average of block i+1)
        // To avoid division, we compare sum1 * count2 > sum2 * count1
        if sum1 * IBig::from(*count2) > sum2 * IBig::from(*count1) {
            // Pool the blocks
            let new_sum = sum1 + sum2;
            let new_count = count1 + count2;
            blocks[i] = (*start1, *end2, new_sum, new_count);
            blocks.remove(i + 1);
            
            // Check if we need to pool with previous blocks
            if i > 0 {
                i -= 1;
            }
        } else {
            i += 1;
        }
    }
    
    // Reconstruct the monotonic sequence
    for (start, end, sum, count) in blocks {
        let avg = sum / IBig::from(count);
        let avg_t = from_ibig_saturating::<T>(avg)?;
        
        for j in start..=end {
            values[j] = avg_t.clone();
        }
    }
    
    // Final pass to ensure strict monotonicity for prefix sums
    // (in case of rounding issues from integer division)
    for i in 1..n {
        if values[i] < values[i - 1] {
            values[i] = values[i - 1].clone();
        }
    }
    
    Ok(values)
}

/// Helper function to compute prefix sum up to time t
fn compute_prefix_sum<T>(data: &[T], t: usize) -> Fallible<IBig>
where
    T: Display,
{
    let mut sum = IBig::zero();
    for i in 0..=t {
        sum += to_ibig(&data[i])?;
    }
    Ok(sum)
}

/// Convert an integer type to IBig
fn to_ibig<T: Display>(value: &T) -> Fallible<IBig> {
    IBig::from_str(&value.to_string())
        .map_err(|_| err!(FailedFunction, "failed to convert to IBig"))
}

/// Convert IBig to an integer type T with saturation
fn from_ibig_saturating<T>(ibig: IBig) -> Fallible<T> 
where 
    T: Integer + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One,
{
    // Try to parse directly
    if let Ok(val) = T::from_str(&ibig.to_string()) {
        return Ok(val);
    }
    
    // If that fails, we need to saturate
    let s = ibig.to_string();
    if s.starts_with('-') {
        // For negative overflow, return minimum value
        if let Some(neg_one) = T::zero().checked_sub(&T::one()) {
            // This is a signed type, find minimum by doubling
            let mut min = neg_one;
            loop {
                if let Some(doubled) = min.checked_add(&min) {
                    min = doubled;
                } else {
                    return Ok(min);
                }
            }
        } else {
            // This is an unsigned type, minimum is zero
            return Ok(T::zero());
        }
    } else {
        // Return maximum value for positive overflow
        let mut max = T::one();
        loop {
            if let Some(doubled) = max.checked_add(&max) {
                if let Some(plus_one) = doubled.checked_add(&T::one()) {
                    max = plus_one;
                } else {
                    return Ok(doubled);
                }
            } else {
                return Ok(max);
            }
        }
    }
}

// Re-export functions that tests need
#[cfg(test)]
use toeplitz_coefficients::{
    compute_optimal_coefficient_scaled as compute_toeplitz_coefficient_scaled,
    compute_inverse_coefficient_scaled,
};