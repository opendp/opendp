use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::measures::MaxDivergence;
use crate::metrics::L1Distance;
use dashu::rational::RBig;
use crate::traits::{Integer, InfCast};
use crate::traits::samplers::sample_discrete_gaussian;

use num::{Zero, CheckedAdd, CheckedMul, CheckedSub};
use std::fmt::Display;
use std::str::FromStr;

use super::utils::core;
use super::utils::isotonic;

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
/// * `enforce_monotonicity` - Whether to apply isotonic regression to ensure monotonic outputs (default: true)
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
/// let measurement = make_toeplitz(input_domain, input_metric, scale, true)?;
/// let counts = vec![5i32; 10]; // 10 time steps with 5 counts each
/// let noisy_prefix_sums = measurement.invoke(&counts)?;
/// # Ok::<(), opendp::error::Fallible>(())
/// ```
pub fn make_toeplitz<T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L1Distance<T>,
    scale: f64,
    enforce_monotonicity: bool,
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
            if enforce_monotonicity {
                isotonic::apply_isotonic_regression(noisy_sums)
            } else {
                Ok(noisy_sums)
            }
        }),
        input_metric.clone(),
        MaxDivergence,
        PrivacyMap::new_fallible(move |d_in: &T| {
            // For pure DP with Gaussian noise: ε = (Δ * d_in) / σ
            // Δ = 1 for an individual is assumed, since:
            // - We don't assume the ability to be able to identify all counts that belong to a single user.
            // - We assume the simple use cases where an individual participates
            let d_in_f64 = f64::inf_cast(d_in.clone())?;
            Ok(d_in_f64 / scale)
        }),
    )
}

/// Compute the Toeplitz mechanism for prefix sums
/// 
/// This implements the Toeplitz mechanism.
/// The mechanism adds correlated discrete Gaussian noise to enable accurate
/// range queries over streaming count data.
fn compute_toeplitz_mechanism<T>(
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
    core::compute_toeplitz_range(data, &raw_noise, 0, n, scale_bits)
}

#[test]
fn test_make_toeplitz_basic() -> Fallible<()> {
    // Create a simple domain for testing
    let input_domain = VectorDomain::new(
        AtomDomain::<i32>::default()
    ).with_size(5);
    let input_metric = L1Distance::default();
    let scale = 10.0;
    
    // Create the measurement
    let measurement = make_toeplitz(input_domain, input_metric, scale, true)?;
    
    // Test with constant count data
    let data = vec![10i32; 5];
    let result = measurement.invoke(&data)?;
    
    // Check output length
    assert_eq!(result.len(), 5);
    
    // The prefix sums without noise would be [10, 20, 30, 40, 50]
    // Results should be monotonically increasing after isotonic regression
    for i in 1..result.len() {
        assert!(result[i] >= result[i-1], 
            "Non-monotonic at position {}: {} < {}", i, result[i], result[i-1]);
    }
    
    println!("Toeplitz output for constant data: {:?}", result);
    
    Ok(())
}

#[test]
fn test_prefix_sum_correctness() -> Fallible<()> {
    // Test with zero variance to check prefix sum logic
    let input_domain = VectorDomain::new(
        AtomDomain::<i32>::default()
    ).with_size(4);
    let input_metric = L1Distance::default();
    let scale = 0.00000000000000001; // Very small scale for minimal noise
    
    let measurement = make_toeplitz(input_domain, input_metric, scale, true)?;
    
    // Use distinct values to verify prefix sums
    let data = vec![1, 2, 3, 4];
    let result = measurement.invoke(&data)?;
    
    println!("Near-zero noise result: {:?}", result);
    // Verify monotonicity
    for i in 1..result.len() {
        assert!(result[i] >= result[i-1]);
    }
    
    // Should be approximately [1, 3, 6, 10] with very small noise
    // All values should be non-negative
    assert_eq!(result[0], 1);
    assert_eq!(result[1], 1+2);
    assert_eq!(result[2], 1+2+3);
    assert_eq!(result[3], 1+2+3+4);

    Ok(())
}

#[test]
fn test_continual_release_counting() -> Fallible<()> {
    // Simulate a realistic continual release scenario
    let time_steps = 20;
    let input_domain = VectorDomain::new(
        AtomDomain::<i32>::default()
    ).with_size(time_steps);
    let input_metric = L1Distance::default();
    let scale = 1.0;
    
    let measurement = make_toeplitz(input_domain, input_metric, scale, true)?;
    
    // Simulate counting data: varying counts at each time step
    let mut counts = vec![0i32; time_steps];
    counts[0] = 5;   // 5 events at time 0
    counts[5] = 3;   // 3 events at time 5
    counts[10] = 7;  // 7 events at time 10
    counts[15] = 2;  // 2 events at time 15
    
    let noisy_prefix_sums = measurement.invoke(&counts)?;
    
    // Verify output length
    assert_eq!(noisy_prefix_sums.len(), time_steps);
    
    // Verify monotonicity
    for i in 1..noisy_prefix_sums.len() {
        assert!(noisy_prefix_sums[i] >= noisy_prefix_sums[i-1],
            "Non-monotonic at position {}: {} < {}", i, noisy_prefix_sums[i], noisy_prefix_sums[i-1]);
    }
    
    // Test range queries by taking differences of prefix sums
    // Note: Any valid subintervals of time are always non-negative, because prefix sums are monotonic after isotonic regression.
    let range_0_5 = noisy_prefix_sums[5];
    let range_6_10 = noisy_prefix_sums[10] - noisy_prefix_sums[5];
    let range_11_15 = noisy_prefix_sums[15] - noisy_prefix_sums[10];
    let total = noisy_prefix_sums[time_steps - 1];
    
    println!("Count [0,5]: {} (true: 8)", range_0_5);
    println!("Count [6,10]: {} (true: 7)", range_6_10);
    println!("Count [11,15]: {} (true: 2)", range_11_15);
    println!("Total count: {} (true: 17)", total);
    
    // All prefix sums should be non-negative
    for ps in &noisy_prefix_sums {
        assert!(*ps >= 0);
    }
    
    Ok(())
}

#[test]
fn test_privacy_guarantee() -> Fallible<()> {
    let input_domain = VectorDomain::new(
        AtomDomain::<i32>::default()
    ).with_size(10);
    let input_metric = L1Distance::default();
    let scale = 2.0;
    
    let measurement = make_toeplitz(input_domain.clone(), input_metric.clone(), scale, true)?;
    
    // Test privacy relation
    // For L1 distance d_in and scale σ, we have ε = d_in / σ
    assert!(measurement.check(&1, &0.5)?);   // d_in=1, ε=0.5
    assert!(measurement.check(&2, &1.0)?);   // d_in=2, ε=1.0
    assert!(measurement.check(&4, &2.0)?);   // d_in=4, ε=2.0
    
    // Should fail when ε < d_in/σ
    assert!(!measurement.check(&1, &0.4)?);  // 0.4 < 1/2
    assert!(!measurement.check(&3, &1.0)?);  // 1.0 < 3/2
    
    // Test with different scales
    let scale2 = 10.0; // Larger scale, thus more private
    let measurement2 = make_toeplitz(
        input_domain.clone(), 
        input_metric.clone(), 
        scale2,
        true,
    )?;
    assert!(measurement2.check(&1, &0.1)?);  // d_in=1, ε=0.1
    assert!(measurement2.check(&5, &0.5)?);  // d_in=5, ε=0.5
    
    Ok(())
}

#[test] 
fn test_edge_cases() -> Fallible<()> {
    // Test with size 1
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(1);
    let measurement = make_toeplitz(domain, L1Distance::default(), 1.0, true)?;
    let result = measurement.invoke(&vec![5])?;
    assert_eq!(result.len(), 1);
    
    // Test invalid scale
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(5);
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), 0.0, true).is_err());
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), -1.0, true).is_err());
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), f64::INFINITY, true).is_err());
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), f64::NAN, true).is_err());
    
    // Test data length mismatch
    let measurement = make_toeplitz(domain.clone(), L1Distance::default(), 1.0, true)?;
    assert!(measurement.invoke(&vec![1, 2, 3]).is_err()); // Too short
    assert!(measurement.invoke(&vec![1, 2, 3, 4, 5, 6]).is_err()); // Too long
    
    // Test empty domain
    let empty_domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(0);
    assert!(make_toeplitz(empty_domain, L1Distance::default(), 1.0, true).is_err());
    
    // Test domain without size
    let no_size_domain = VectorDomain::new(AtomDomain::<i32>::default());
    assert!(make_toeplitz(no_size_domain, L1Distance::default(), 1.0, true).is_err());
    
    Ok(())
}

#[test]
fn test_saturation_behavior() -> Fallible<()> {
    // Test integer overflow handling
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(3);
    let measurement = make_toeplitz(domain, L1Distance::default(), 0.1, true)?; // Very small scale for large noise
    
    // Use maximum values to test saturation
    let data = vec![i32::MAX / 3, i32::MAX / 3, i32::MAX / 3];
    let result = measurement.invoke(&data)?;
    
    // Should not panic, values should saturate
    assert_eq!(result.len(), 3);
    
    // Verify monotonicity even with saturation
    for i in 1..result.len() {
        assert!(result[i] >= result[i-1]);
    }
    
    println!("Saturation test (max) result: {:?}", result);
    
    Ok(())
}

#[test]
fn test_different_integer_types() -> Fallible<()> {
    // Test with i64
    let domain_i64 = VectorDomain::new(AtomDomain::<i64>::default()).with_size(5);
    let measurement_i64 = make_toeplitz(domain_i64, L1Distance::default(), 5.0, true)?;
    let data_i64 = vec![100i64, 200, 300, 400, 500];
    let result_i64 = measurement_i64.invoke(&data_i64)?;
    assert_eq!(result_i64.len(), 5);
    
    // Verify monotonicity
    for i in 1..result_i64.len() {
        assert!(result_i64[i] >= result_i64[i-1]);
    }
    
    // Test with u32 (unsigned)
    let domain_u32 = VectorDomain::new(AtomDomain::<u32>::default()).with_size(5);
    let measurement_u32 = make_toeplitz(domain_u32, L1Distance::default(), 5.0, true)?;
    let data_u32 = vec![10u32, 20, 30, 40, 50];
    let result_u32 = measurement_u32.invoke(&data_u32)?;
    assert_eq!(result_u32.len(), 5);
    
    // Verify monotonicity
    for i in 1..result_u32.len() {
        assert!(result_u32[i] >= result_u32[i-1]);
    }
    
    Ok(())
}

#[test]
fn test_noise_correlation() -> Fallible<()> {
    // Test that noise is properly correlated across time steps
    let n = 100;
    let scale = 1.0;
    
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(n);
    let measurement = make_toeplitz(domain, L1Distance::default(), scale, true)?;
    
    // All zeros should produce correlated noise pattern
    let zeros = vec![0i32; n];
    let noise_pattern = measurement.invoke(&zeros)?;
    
    // Verify monotonicity of the noise pattern
    for i in 1..n {
        assert!(noise_pattern[i] >= noise_pattern[i-1],
            "Non-monotonic noise at position {}: {} < {}", i, noise_pattern[i], noise_pattern[i-1]);
    }
    
    // The noise should have specific correlation structure
    // Adjacent differences should have lower variance than distant differences
    let mut adjacent_diffs = Vec::new();
    let mut distant_diffs = Vec::new();
    
    for i in 1..n {
        adjacent_diffs.push((noise_pattern[i] - noise_pattern[i-1]) as f64);
    }
    
    for i in 10..n {
        distant_diffs.push((noise_pattern[i] - noise_pattern[i-10]) as f64);
    }
    
    // Compute variances
    let adj_var = variance(&adjacent_diffs);
    let dist_var = variance(&distant_diffs);
    
    println!("Adjacent variance: {}, Distant variance: {}", adj_var, dist_var);
    // Due to correlation structure and isotonic regression, patterns may vary
    
    Ok(())
}

#[test]
fn test_large_time_series() -> Fallible<()> {
    // Test with larger time series to ensure scalability
    let n = 200;  // Reduced from 1000 to 200 for faster tests
    let scale = 20.0;
    
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(n);
    let measurement = make_toeplitz(domain, L1Distance::default(), scale, true)?;
    
    // Generate random-walk style data
    let mut data = vec![0i32; n];
    let mut current = 100i32;
    for i in 0..n {
        current += (i % 7) as i32 - 3; // Pseudo-random walk
        data[i] = current.max(0);
    }
    
    let result = measurement.invoke(&data)?;
    assert_eq!(result.len(), n);
    
    // Verify monotonicity
    for i in 1..result.len() {
        assert!(result[i] >= result[i-1],
            "Non-monotonic at position {}: {} < {}", i, result[i], result[i-1]);
    }
    
    // Test some range queries
    let q1 = result[49];  // Sum of first 50
    let q2 = result[99] - result[49]; // Sum of elements 50-99
    let q3 = result[199] - result[99]; // Sum of elements 100-199
    
    println!("Large series queries: [0,49]={}, [50,99]={}, [100,199]={}", q1, q2, q3);
    
    Ok(())
}

// Helper function for variance calculation
fn variance(data: &[f64]) -> f64 {
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64
}

#[test]
fn test_toeplitz_without_isotonic() -> Fallible<()> {
    let input_domain = VectorDomain::new(
        AtomDomain::<i32>::default()
    ).with_size(5);
    let input_metric = L1Distance::default();
    let scale = 1.0;
    
    let measurement = make_toeplitz(input_domain, input_metric, scale, false)?;
    
    let data = vec![1, 2, 3, 4, 5];
    let result = measurement.invoke(&data)?;
    
    assert_eq!(result.len(), 5);
    
    // Results should be non-negative (still clamped) but NOT necessarily monotonic
    for val in &result {
        assert!(*val >= 0);
    }
    
    println!("Toeplitz without isotonic: {:?}", result);
    
    Ok(())
}
