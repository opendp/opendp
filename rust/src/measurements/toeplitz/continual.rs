use crate::error::*;
use crate::traits::{Integer, InfCast};
use crate::traits::samplers::sample_discrete_gaussian;
use dashu::rational::RBig;
use dashu::integer::IBig;
use std::sync::{Arc, Mutex};
use std::fmt::Display;
use std::str::FromStr;
use num::{CheckedAdd, CheckedMul, CheckedSub, Zero};

use super::{compute_toeplitz_range, to_ibig, from_ibig_saturating};
use super::isotonic::{apply_isotonic_regression, apply_isotonic_regression_with_fixed_prefix};

/// Stateful container for continual release with the Toeplitz mechanism
pub struct ContinualToeplitz<T> {
    /// Scale parameter (\sigma). In the context of approximate DP, the privacy loss is
    /// $\epsilon = \Delta \sqrt{2 \ln( 1.25/ \delta )} / \sigma$, where $\Delta$ is the
    /// sensitivity of the query, and the privacy guarantee is $(\epsilon, \delta)$-DP.
    /// So, $\sigma$ is set at a trade-off with the privacy loss.
    scale: f64,
    /// Precision bits for coefficient calculations
    scale_bits: usize,
    /// Storage for raw noise values Z[t]
    raw_noise_history: Arc<Mutex<Vec<IBig>>>,
    /// Storage for already-computed noisy prefix sums after isotonic regression
    noisy_prefix_sums: Arc<Mutex<Vec<T>>>,
    /// Variance for noise generation
    variance: RBig,
    /// Whether to enforce monotonicity
    enforce_monotonicity: bool,
}

impl<T> ContinualToeplitz<T> 
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd + Clone,
{
    /// Create a new continual Toeplitz mechanism
    pub fn new(scale: f64, enforce_monotonicity: bool) -> Fallible<Self> {
        if scale.is_sign_negative() || scale.is_zero() || !scale.is_finite() {
            return fallible!(MakeMeasurement, "scale must be positive and finite");
        }
        
        let variance = RBig::from((scale * scale * 1e9) as i64) / RBig::from(1_000_000_000i64);
        
        Ok(ContinualToeplitz {
            scale,
            scale_bits: 40,  // 40 bits provides ~12 decimal digits of precision
            raw_noise_history: Arc::new(Mutex::new(Vec::new())),
            noisy_prefix_sums: Arc::new(Mutex::new(Vec::new())),
            variance,
            enforce_monotonicity,
        })
    }
    
    /// Process a new batch of counts up to time `new_time`
    /// 
    /// # Arguments
    /// * `counts` - Full count vector from time 0 to new_time (exclusive)
    /// 
    /// # Returns
    /// All noisy prefix sums from time 0 to new_time (exclusive), guaranteed to be monotonic
    /// through isotonic regression post-processing if enforce_monotonicity is true
    pub fn release(&self, counts: &[T]) -> Fallible<Vec<T>> {
        let new_time = counts.len();
        if new_time == 0 {
            return Ok(vec![]);
        }
        
        let mut raw_noise_history = self.raw_noise_history.lock().unwrap();
        let mut noisy_prefix_sums = self.noisy_prefix_sums.lock().unwrap();
        
        let current_time = noisy_prefix_sums.len();
        
        // Validate that time is non-decreasing
        if new_time < current_time {
            return fallible!(
                FailedFunction, 
                "new time {} is less than previous maximum time {}", 
                new_time, 
                current_time
            );
        }
        
        // If querying the same time range, return cached results
        if new_time == current_time {
            return Ok(noisy_prefix_sums.clone());
        }
        
        // Generate new raw noise for time steps [current_time, new_time)
        for _ in current_time..new_time {
            raw_noise_history.push(sample_discrete_gaussian(self.variance.clone())?);
        }
        
        // Compute ALL noisy prefix sums from scratch to ensure consistency
        // This is necessary because the Toeplitz mechanism's correlated noise structure
        // means that noise at time t depends on all previous raw noise values
        let mut all_noisy_sums = compute_toeplitz_range(
            counts,
            &raw_noise_history,
            0,
            new_time,
            self.scale_bits
        )?;
        
        // Clamp to non-negative before isotonic regression
        for i in 0..all_noisy_sums.len() {
            if all_noisy_sums[i] < T::zero() {
                all_noisy_sums[i] = T::zero();
            }
        }
        
        // Apply isotonic regression with constraint to preserve previously released values
        let monotonic_sums = if self.enforce_monotonicity {
            if current_time == 0 {
                // First release: standard isotonic regression
                apply_isotonic_regression(all_noisy_sums)?
            } else {
                // Subsequent releases: preserve the first current_time values
                let fixed_prefix = noisy_prefix_sums[..current_time].to_vec();
                apply_isotonic_regression_with_fixed_prefix(all_noisy_sums, fixed_prefix)?
            }
        } else {
            all_noisy_sums
        };
        
        // Update the stored results
        *noisy_prefix_sums = monotonic_sums.clone();
        
        Ok(monotonic_sums)
    }
    
    /// Get the privacy cost for a given input distance
    pub fn privacy_cost(&self, d_in: T) -> Fallible<f64> 
    where
        f64: InfCast<T>,
    {
        let d_in_f64 = f64::inf_cast(d_in)?;
        Ok(d_in_f64 / self.scale)
    }
}

/// Create a measurement for continual release using the Toeplitz mechanism
/// 
/// This maintains state across invocations to ensure consistency of noise and
/// guarantees monotonic outputs through isotonic regression post-processing.
pub fn make_continual_toeplitz<T>(
    scale: f64,
    enforce_monotonicity: bool,
) -> Fallible<impl Fn(&Vec<T>) -> Fallible<Vec<T>>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd + Clone + Send + Sync + 'static,
    f64: InfCast<T>,
{
    let mechanism = ContinualToeplitz::new(scale, enforce_monotonicity)?;
    let mechanism = Arc::new(mechanism);
    
    Ok(move |counts: &Vec<T>| {
        mechanism.release(counts)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_continual_consistency() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?; // enforce monotonicity
        
        // First query: times [0, 5)
        let counts1 = vec![10, 20, 30, 40, 50];
        let result1 = mechanism.release(&counts1)?;
        assert_eq!(result1.len(), 5);
        
        // Verify monotonicity
        for i in 1..result1.len() {
            assert!(result1[i] >= result1[i-1], 
                "Non-monotonic at position {}: {} < {}", i, result1[i], result1[i-1]);
        }
        
        println!("First result: {:?}, with the real counts being: {:?}", result1, counts1);
        
        // Second query: times [0, 8)
        let counts2 = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let result2 = mechanism.release(&counts2)?;
        assert_eq!(result2.len(), 8);
        
        // Verify monotonicity
        for i in 1..result2.len() {
            assert!(result2[i] >= result2[i-1], 
                "Non-monotonic at position {}: {} < {}", i, result2[i], result2[i-1]);
        }
        
        println!("Second result: {:?}, with the real counts being: {:?}", result2, counts2);
        
        // The first 5 values should be identical (consistency across releases)
        for i in 0..5 {
            assert_eq!(result1[i], result2[i], "Inconsistent value at time {}", i);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_continual_decreasing_time_error() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        
        // First query: times [0, 5)
        let counts1 = vec![10, 20, 30, 40, 50];
        mechanism.release(&counts1)?;
        
        // Second query with fewer time steps should fail
        let counts2 = vec![10, 20, 30];
        assert!(mechanism.release(&counts2).is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_continual_same_query_returns_same_result() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        
        // First query
        let counts = vec![10, 20, 30, 40, 50];
        let result1 = mechanism.release(&counts)?;
        
        // Same query should return same result
        let result2 = mechanism.release(&counts)?;
        
        assert_eq!(result1, result2);
        
        Ok(())
    }
    
    #[test]
    fn test_monotonicity_enforcement() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(1.0, true)?; // Small scale for more noise
        
        // Generate many releases to test monotonicity
        for _ in 0..10 {
            let counts = vec![5, 10, 15, 20, 25, 30, 35, 40];
            let result = mechanism.release(&counts)?;
            
            // Check monotonicity
            for i in 1..result.len() {
                assert!(result[i] >= result[i-1], 
                    "Monotonicity violation at {}: {} < {}", i, result[i], result[i-1]);
            }
            
            // Verify non-negative (prefix sums should never be negative)
            for (i, &val) in result.iter().enumerate() {
                assert!(val >= 0, "Negative value at position {}: {}", i, val);
            }
        }
        
        Ok(())
    }
    
    #[test]
    fn test_continual_without_monotonicity() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, false)?; // no monotonicity
        
        let counts = vec![5, 10, 15, 20, 25];
        let result = mechanism.release(&counts)?;
        assert_eq!(result.len(), 5);
        
        // Just verify non-negative (still clamped)
        for val in &result {
            assert!(*val >= 0);
        }
        
        // Not checking monotonicity since we disabled it
        println!("Continual without monotonicity: {:?}", result);
        
        Ok(())
    }
}
