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

use super::core::compute_toeplitz_range;
use super::isotonic::apply_isotonic_regression;

/// Create a baseline Toeplitz measurement (no post-processing)
/// 
/// This returns a measurement directly without unnecessary wrapper structs.
/// The baseline version does not enforce monotonicity in the output prefix sums.
/// 
/// # Arguments
/// * `input_domain` - VectorDomain with known size containing the time series of counts
/// * `input_metric` - L1Distance metric for measuring sensitivity  
/// * `scale` - Noise scale parameter (σ)
/// 
/// # Returns
/// A Measurement that computes differentially private prefix sums without monotonicity enforcement
pub fn make_baseline_toeplitz<T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L1Distance<T>,
    scale: f64,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L1Distance<T>, MaxDivergence>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd,
    f64: InfCast<T>,
{
    make_toeplitz(input_domain, input_metric, scale, false)
}

/// Create a monotonic Toeplitz measurement (with isotonic regression)
/// 
/// This returns a measurement directly without unnecessary wrapper structs.
/// The monotonic version enforces non-decreasing prefix sums through isotonic regression.
/// 
/// # Arguments
/// * `input_domain` - VectorDomain with known size containing the time series of counts
/// * `input_metric` - L1Distance metric for measuring sensitivity  
/// * `scale` - Noise scale parameter (σ)
/// 
/// # Returns
/// A Measurement that computes differentially private prefix sums with monotonicity enforcement
pub fn make_monotonic_toeplitz<T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L1Distance<T>,
    scale: f64,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L1Distance<T>, MaxDivergence>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd,
    f64: InfCast<T>,
{
    make_toeplitz(input_domain, input_metric, scale, true)
}

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
                return fallible!(
                    FailedFunction,
                    "expected data of length {}, got {}",
                    n,
                    data.len()
                );
            }
            
            // Convert scale to variance for discrete Gaussian
            let variance = RBig::from((scale * scale * 1e9) as i64) / RBig::from(1_000_000_000i64);
            
            // Step 1: Generate independent discrete Gaussian noise Z
            let mut raw_noise = Vec::with_capacity(n);
            for _ in 0..n {
                raw_noise.push(sample_discrete_gaussian(variance.clone())?);
            }
            
            // Step 2: Apply Toeplitz mechanism to compute noisy prefix sums
            let mut noisy_sums = compute_toeplitz_range(data, &raw_noise, 0, n, scale_bits)?;
            
            // Apply isotonic regression if requested
            if enforce_monotonicity {
                noisy_sums = apply_isotonic_regression(noisy_sums)?;
            }
            
            Ok(noisy_sums)
        }),
        input_metric.clone(),
        MaxDivergence,
        PrivacyMap::new_fallible(move |d_in: &T| -> Fallible<f64> {
            // Privacy guarantee: ε = d_in / scale under pure DP
            let d_in_f64 = f64::inf_cast(d_in.clone())?;
            Ok(d_in_f64 / scale)
        }),
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monotonic_enforces_increasing() -> Fallible<()> {
        let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(10);
        let measurement = make_monotonic_toeplitz(domain, L1Distance::default(), 5.0)?;
        
        let data = vec![1; 10];
        let result = measurement.invoke(&data)?;
        
        // Verify monotonicity
        for i in 1..result.len() {
            assert!(result[i] >= result[i-1]);
        }
        Ok(())
    }
    
    #[test]
    fn test_baseline_allows_decrease() -> Fallible<()> {
        let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(5);
        let measurement = make_baseline_toeplitz(domain, L1Distance::default(), 1.0)?;
        
        let data = vec![1, 2, 3, 4, 5];
        let result = measurement.invoke(&data)?;
        
        assert_eq!(result.len(), 5);
        // Baseline doesn't enforce monotonicity
        Ok(())
    }
    
    #[test]
    fn test_privacy_map() -> Fallible<()> {
        let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(5);
        let measurement = make_toeplitz(domain, L1Distance::default(), 10.0, true)?;
        
        assert_eq!(measurement.map(&5)?, 0.5);   // ε = 5/10
        assert_eq!(measurement.map(&20)?, 2.0);  // ε = 20/10
        Ok(())
    }
    
    #[test]
    fn test_size_one() -> Fallible<()> {
        let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(1);
        let measurement = make_monotonic_toeplitz(domain, L1Distance::default(), 1.0)?;
        
        let result = measurement.invoke(&vec![5])?;
        assert_eq!(result.len(), 1);
        Ok(())
    }
    
    #[test]
    fn test_different_types() -> Fallible<()> {
        // i64
        let domain = VectorDomain::new(AtomDomain::<i64>::default()).with_size(3);
        let meas = make_baseline_toeplitz(domain, L1Distance::default(), 5.0)?;
        assert_eq!(meas.invoke(&vec![1i64, 2, 3])?.len(), 3);
        
        // u32
        let domain = VectorDomain::new(AtomDomain::<u32>::default()).with_size(3);
        let meas = make_monotonic_toeplitz(domain, L1Distance::default(), 5.0)?;
        assert_eq!(meas.invoke(&vec![1u32, 2, 3])?.len(), 3);
        Ok(())
    }
    
    #[test]
    fn test_zero_input_produces_noise() -> Fallible<()> {
        let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(10);
        let measurement = make_monotonic_toeplitz(domain, L1Distance::default(), 100.0)?;
        
        let zeros = vec![0i32; 10];
        let result = measurement.invoke(&zeros)?;
        
        // Should have some non-zero noise
        assert!(result.iter().any(|&x| x != 0));
        Ok(())
    }
}
