use crate::error::*;
use crate::traits::{Integer, InfCast};
use crate::traits::samplers::sample_discrete_gaussian;
use dashu::rational::RBig;
use dashu::integer::IBig;
use std::sync::Mutex;
use std::fmt::Display;
use std::str::FromStr;
use num::{CheckedAdd, CheckedMul, CheckedSub, Zero};

use super::utils::core::compute_toeplitz_range;
use super::utils::isotonic::{apply_isotonic_regression, apply_isotonic_regression_with_fixed_prefix};

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
    /// Information that can change over time is stored in a mutex
    /// to allow safe concurrent access
    state: Mutex<ToeplitzState<T>>,
    /// Information that are immutable for the lifetime of the mechanism are stored in a config
    config: ToeplitzConfig,
}

/// Mutable state for the Toeplitz mechanism
struct ToeplitzState<T> {
    /// Raw noise values Z[t] as a single vector
    raw_noise_history: Vec<IBig>,
    /// Already-computed noisy prefix sums after isotonic regression as a vector
    noisy_prefix_sums: Vec<T>,
    /// Cumulative counts from all previous releases as a vector
    cumulative_counts: Vec<T>,
}

/// Immutable configuration for the Toeplitz mechanism
struct ToeplitzConfig {
    /// Scale parameter (σ)
    scale: f64,
    /// Precision bits for coefficient calculations
    scale_bits: usize,
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
            state: Mutex::new(ToeplitzState {
                raw_noise_history: Vec::new(),
                noisy_prefix_sums: Vec::new(),
                cumulative_counts: Vec::new(),
            }),
            config: ToeplitzConfig {
                scale,
                scale_bits: 40,  // 40 bits provides ~12 decimal digits of precision
                variance,
                enforce_monotonicity,
            },
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
        let mut state = self.state.lock().unwrap();
        
        let current_time = state.cumulative_counts.len();
        
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
            return Ok(state.noisy_prefix_sums.clone());
        }
        
        // Append incremental counts to cumulative history
        state.cumulative_counts.extend_from_slice(incremental_counts);
        let new_time = state.cumulative_counts.len();
        
        // Generate new raw noise for time steps [current_time, new_time)
        for _ in current_time..new_time {
            state.raw_noise_history.push(sample_discrete_gaussian(self.config.variance.clone())?);
        }
        
        // Compute ALL noisy prefix sums from scratch to ensure consistency
        // This is necessary because the Toeplitz mechanism's correlated noise structure
        // means that noise at time t depends on all previous raw noise values
        let mut all_noisy_sums = compute_toeplitz_range(
            &state.cumulative_counts,
            &state.raw_noise_history,
            0,
            new_time,
            self.config.scale_bits
        )?;
        
        // Clamp to non-negative before isotonic regression
        for i in 0..all_noisy_sums.len() {
            if all_noisy_sums[i] < T::zero() {
                all_noisy_sums[i] = T::zero();
            }
        }
        
        // Apply isotonic regression with constraint to preserve previously released values
        let monotonic_sums = if self.config.enforce_monotonicity {
            if current_time == 0 {
                // First release: standard isotonic regression
                apply_isotonic_regression(all_noisy_sums)?
            } else {
                // Subsequent releases: preserve the first current_time values
                let fixed_prefix = state.noisy_prefix_sums[..current_time].to_vec();
                apply_isotonic_regression_with_fixed_prefix(all_noisy_sums, fixed_prefix)?
            }
        } else {
            all_noisy_sums
        };
        
        // Update the stored results
        state.noisy_prefix_sums = monotonic_sums.clone();
        
        Ok(monotonic_sums)
    }
    
    /// Get the current number of time steps processed
    pub fn current_time(&self) -> usize {
        self.state.lock().unwrap().cumulative_counts.len()
    }
    
    /// Get the privacy cost for a given input distance
    pub fn privacy_cost(&self, d_in: T) -> Fallible<f64> 
    where
        f64: InfCast<T>,
    {
        let d_in_f64 = f64::inf_cast(d_in)?;
        Ok(d_in_f64 / self.config.scale)
    }
}

/// Baseline continual Toeplitz (no monotonicity enforcement)
pub struct BaselineContinualToeplitz<T: crate::traits::CheckAtom>(ContinualToeplitz<T>);

impl<T: crate::traits::CheckAtom> BaselineContinualToeplitz<T>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd + Clone,
{
    pub fn new(scale: f64) -> Fallible<Self> {
        Ok(Self(ContinualToeplitz::new(scale, false)?))
    }
    
    pub fn release(&self, incremental_counts: &[T], expected_previous_time: usize) -> Fallible<Vec<T>> {
        self.0.release(incremental_counts, expected_previous_time)
    }
    
    pub fn current_time(&self) -> usize {
        self.0.current_time()
    }
    
    pub fn privacy_cost(&self, d_in: T) -> Fallible<f64> 
    where
        f64: InfCast<T>,
    {
        self.0.privacy_cost(d_in)
    }
}

/// Monotonic continual Toeplitz (with isotonic regression)
pub struct MonotonicContinualToeplitz<T: crate::traits::CheckAtom>(ContinualToeplitz<T>);

impl<T: crate::traits::CheckAtom> MonotonicContinualToeplitz<T>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd + Clone,
{
    pub fn new(scale: f64) -> Fallible<Self> {
        Ok(Self(ContinualToeplitz::new(scale, true)?))
    }
    
    pub fn release(&self, incremental_counts: &[T], expected_previous_time: usize) -> Fallible<Vec<T>> {
        self.0.release(incremental_counts, expected_previous_time)
    }
    
    pub fn current_time(&self) -> usize {
        self.0.current_time()
    }
    
    pub fn privacy_cost(&self, d_in: T) -> Fallible<f64> 
    where
        f64: InfCast<T>,
    {
        self.0.privacy_cost(d_in)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_continual_consistency() -> Fallible<()> {
        // let mechanism = ContinualToeplitz::<i32>::new(1.0, true)?;
        let mechanism = MonotonicContinualToeplitz::<i32>::new(1.0)?;  // enforce monotonicity
        
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
        // let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
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
        // let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
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
        // let mechanism = ContinualToeplitz::<i32>::new(1.0, true)?;
        let mechanism = MonotonicContinualToeplitz::<i32>::new(1.0)?;  // Small scale for more noise
        
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
        // let mechanism = ContinualToeplitz::<i32>::new(10.0, false)?;
        let mechanism = BaselineContinualToeplitz::<i32>::new(10.0)?;  // no monotonicity
        
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
        // let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
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
        // let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
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

    #[test]
    fn test_baseline_current_time() -> Fallible<()> {
        let mechanism = BaselineContinualToeplitz::<i32>::new(10.0)?;
        
        assert_eq!(mechanism.current_time(), 0);
        
        mechanism.release(&vec![1, 2, 3], 0)?;
        assert_eq!(mechanism.current_time(), 3);
        
        mechanism.release(&vec![4, 5], 3)?;
        assert_eq!(mechanism.current_time(), 5);
        
        Ok(())
    }

    #[test]
    fn test_privacy_cost_calculation() -> Fallible<()> {
        // This test ensures the scale field is recognized as used
        // let mechanism = ContinualToeplitz::<i32>::new(10.0, true)?;
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
        // Verify privacy cost calculation: ε = d_in / scale
        assert_eq!(mechanism.privacy_cost(1)?, 0.1);
        assert_eq!(mechanism.privacy_cost(10)?, 1.0);
        assert_eq!(mechanism.privacy_cost(20)?, 2.0);
        
        // Different scale
        // let mechanism2 = ContinualToeplitz::<i64>::new(5.0, false)?;
        let mechanism2 = BaselineContinualToeplitz::<i64>::new(5.0)?;
        assert_eq!(mechanism2.privacy_cost(5)?, 1.0);
        assert_eq!(mechanism2.privacy_cost(10)?, 2.0);
        
        Ok(())
    }
}
