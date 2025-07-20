use crate::error::*;
use crate::traits::{Integer, InfCast};
use crate::traits::samplers::sample_discrete_gaussian;
use dashu::rational::RBig;
use dashu::integer::IBig;
use std::sync::{Arc, Mutex};
use std::fmt::Display;
use std::str::FromStr;
use num::{CheckedAdd, CheckedMul, CheckedSub, Zero};

use super::compute_toeplitz_range;
use super::isotonic::{apply_isotonic_regression, apply_isotonic_regression_with_fixed_prefix};

/// Stateful container for continual release with the Toeplitz mechanism
/// 
/// ContinualToeplitz maintains state across multiple releases to ensure consistency
/// of noise. The API only accepts incremental counts since the last release,
/// eliminating redundant data transmission.
/// 
/// # Example
/// ```
/// let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
/// 
/// // First release: provide initial counts, expect 0 previous time steps
/// let result1 = mechanism.release(&vec![10, 20, 30], 0)?;
/// 
/// // Second release: provide ONLY new counts, expect 3 previous time steps  
/// let result2 = mechanism.release(&vec![40, 50], 3)?;
/// 
/// // Query current state without adding new data
/// let result3 = mechanism.release(&vec![], 5)?;
/// ```
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
    /// Storage for cumulative counts from all previous releases
    cumulative_counts: Arc<Mutex<Vec<T>>>,
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
            cumulative_counts: Arc::new(Mutex::new(Vec::new())),
            variance,
            enforce_monotonicity,
        })
    }
    
    /// Process incremental counts since the last release
    /// 
    /// # Arguments
    /// * `incremental_counts` - New counts since the last release (not the full history)
    /// * `expected_previous_time` - Expected number of time steps already processed (for verification)
    /// 
    /// # Returns
    /// All noisy prefix sums from time 0 up to and including the new counts,
    /// guaranteed to be monotonic through isotonic regression post-processing if enforce_monotonicity is true
    /// 
    /// # Errors
    /// Returns an error if `expected_previous_time` doesn't match the actual state
    pub fn release(&self, incremental_counts: &[T], expected_previous_time: usize) -> Fallible<Vec<T>> {
        let mut raw_noise_history = self.raw_noise_history.lock().unwrap();
        let mut noisy_prefix_sums = self.noisy_prefix_sums.lock().unwrap();
        let mut cumulative_counts = self.cumulative_counts.lock().unwrap();
        
        let current_time = cumulative_counts.len();
        
        // Verify expected state
        if expected_previous_time != current_time {
            return fallible!(
                FailedFunction,
                "expected previous time {} does not match actual time {}",
                expected_previous_time,
                current_time
            );
        }
        
        // If no new counts, return existing results
        if incremental_counts.is_empty() {
            return Ok(noisy_prefix_sums.clone());
        }
        
        // Append incremental counts to cumulative history
        cumulative_counts.extend_from_slice(incremental_counts);
        let new_time = cumulative_counts.len();
        
        // Generate new raw noise for time steps [current_time, new_time)
        for _ in current_time..new_time {
            raw_noise_history.push(sample_discrete_gaussian(self.variance.clone())?);
        }
        
        // Compute ALL noisy prefix sums from scratch to ensure consistency
        // This is necessary because the Toeplitz mechanism's correlated noise structure
        // means that noise at time t depends on all previous raw noise values
        let mut all_noisy_sums = compute_toeplitz_range(
            &cumulative_counts,
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
    
    /// Get the current number of time steps processed
    pub fn current_time(&self) -> usize {
        self.cumulative_counts.lock().unwrap().len()
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
) -> Fallible<impl Fn(&Vec<T>, usize) -> Fallible<Vec<T>>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd + Clone + Send + Sync + 'static,
    f64: InfCast<T>,
{
    let mechanism = ContinualToeplitz::new(scale, enforce_monotonicity)?;
    let mechanism = Arc::new(mechanism);
    
    Ok(move |incremental_counts: &Vec<T>, expected_previous_time: usize| {
        mechanism.release(incremental_counts, expected_previous_time)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_continual_consistency() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(1.0, true)?; // enforce monotonicity
        
        // First query: times [0, 5)
        let counts1 = vec![10, 20, 30, 40, 50];
        let result1 = mechanism.release(&counts1, 0)?;
        assert_eq!(result1.len(), 5);
        
        // Verify monotonicity
        for i in 1..result1.len() {
            assert!(result1[i] >= result1[i-1], 
                "Non-monotonic at position {}: {} < {}", i, result1[i], result1[i-1]);
        }
        
        println!("First result: {:?}, with the real counts being: {:?}", result1, counts1);
        
        // Second query: times [5, 8)
        let counts2 = vec![60, 70, 80];
        let result2 = mechanism.release(&counts2, 5)?;
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
    fn test_continual_wrong_expected_time_error() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        
        // First query
        let counts1 = vec![10, 20, 30, 40, 50];
        mechanism.release(&counts1, 0)?;
        
        // Second query with wrong expected time should fail
        let counts2 = vec![60, 70, 80];
        let result = mechanism.release(&counts2, 3); // Wrong! Should be 5
        
        assert!(result.is_err());
        // Just verify it's an error - the exact message format varies
        
        Ok(())
    }
    
    #[test]
    fn test_continual_same_query_returns_different_result() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        
        // First query
        let counts = vec![10, 20, 30, 40, 50];
        let result1 = mechanism.release(&counts, 0)?;
        
        // Same counts added again should give different result (different time range)
        let result2 = mechanism.release(&counts, 5)?;
        
        assert_ne!(result1, result2);
        assert_eq!(result2.len(), 10);
        
        Ok(())
    }
    
    #[test]
    fn test_monotonicity_enforcement() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(1.0, true)?; // Small scale for more noise
        
        // Initial release
        let counts = vec![5, 10, 15, 20];
        let result1 = mechanism.release(&counts, 0)?;
        
        // Check monotonicity
        for i in 1..result1.len() {
            // Monotonicity here is not enforced by default
            // It is enforced because mechanism is instantiasted to respect monotonicity,
            // with the corresponding parameter set to true.
            assert!(result1[i] >= result1[i-1], 
                "Monotonicity violation at {}: {} < {}", i, result1[i], result1[i-1]);
        }
        
        // Second release
        let counts2 = vec![25, 30, 35, 40];
        let result2 = mechanism.release(&counts2, 4)?;
        
        // Check monotonicity for full result
        for i in 1..result2.len() {
            // Monotonicity here is not enforced by default
            // It is enforced because mechanism is instantiasted to respect monotonicity,
            // with the corresponding parameter set to true.
            assert!(result2[i] >= result2[i-1], 
                "Monotonicity violation at {}: {} < {}", i, result2[i], result2[i-1]);
        }
        
        // Verify non-negative (prefix sums should never be negative)
        for (i, &val) in result2.iter().enumerate() {
            assert!(val >= 0, "Negative value at position {}: {}", i, val);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_continual_without_monotonicity() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, false)?; // no monotonicity
        
        let counts = vec![5, 10, 15, 20, 25];
        let result = mechanism.release(&counts, 0)?;
        assert_eq!(result.len(), 5);
        
        // Just verify non-negative (still clamped)
        for val in &result {
            assert!(*val >= 0);
        }
        
        // Not checking monotonicity since we disabled it
        println!("Continual without monotonicity: {:?}", result);
        
        Ok(())
    }
    
    #[test]
    fn test_empty_incremental_counts() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        
        // First release with some data
        let counts1 = vec![10, 20, 30];
        let result1 = mechanism.release(&counts1, 0)?;
        assert_eq!(result1.len(), 3);
        
        // Release with empty incremental counts - should return same results
        let result2 = mechanism.release(&vec![], 3)?;
        assert_eq!(result2.len(), 3);
        assert_eq!(result1, result2); // Should be identical
        
        Ok(())
    }
    
    #[test]
    fn test_state_preservation_on_error() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        
        // First successful release
        let counts1 = vec![10, 20, 30];
        let result1 = mechanism.release(&counts1, 0)?;
        assert_eq!(result1.len(), 3);
        
        // Failed release (wrong expected time)
        let counts2 = vec![40, 50];
        let _err = mechanism.release(&counts2, 10).unwrap_err(); // Should fail
        
        // Verify state wasn't modified - check using current_time
        assert_eq!(mechanism.current_time(), 3);
        
        // Successful release with correct expected time should work
        let result3 = mechanism.release(&counts2, 3)?;
        assert_eq!(result3.len(), 5);
        
        Ok(())
    }
}
