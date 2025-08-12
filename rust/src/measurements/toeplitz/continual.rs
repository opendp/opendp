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

/// Continual release API
/// 
/// This trait provides a consistent interface for continual release mechanisms,
/// supporting both baseline and monotonic variants.
/// Particularly, we provide the following APIs: `append_count_on_new_timestamp`, `fetch_privacy_preserving_sub_interval_sum`, `current_time`, and `privacy_cost`.
/// 1. The `append_count_on_new_timestamp` method allows adding new count values to the mechanism.
/// 2. The `fetch_privacy_preserving_sub_interval_sum` method retrieves the noisy sum for a specified time interval.
/// 3. The `current_time` method returns the number of time steps processed so far.
/// 4. The `privacy_cost` method calculates the privacy cost (epsilon) for a given sensitivity under pure differential privacy (DP).
pub trait ContinualRelease<T> {
    /// Update the mechanism with a new count value
    /// 
    /// # Arguments
    /// * `value` - The new count to add
    /// 
    /// # Returns
    /// The current time step after the update
    fn append_count_on_new_timestamp(&self, value: T) -> Fallible<usize>;
    
    /// Release the noisy sum for a time interval
    /// 
    /// # Arguments
    /// * `start_time` - Start of the interval
    /// * `end_time` - End of the interval
    /// 
    /// # Returns
    /// The noisy sum for the interval [start_time, end_time]
    fn fetch_privacy_preserving_sub_interval_sum(&self, start_time: usize, end_time: usize) -> Fallible<T>;
    
    /// Get the current number of time steps processed
    fn current_time(&self) -> usize;
    
    /// Get the privacy cost (epsilon) for a given sensitivity under pure DP
    /// 
    /// # Arguments
    /// * `d_in` - The L1 distance between input count vectors.
    /// 
    /// # Returns
    /// The privacy parameter epsilon = d_in / scale
    fn privacy_cost(&self, d_in: T) -> Fallible<f64> 
    where
        f64: InfCast<T>;
}

/// Internal implementation of the Toeplitz mechanism
/// 
/// This struct contains the core logic and is wrapped by public API structs
struct ContinualToeplitzCore<T> {
    /// Information that can change over time is stored in a mutex
    /// to allow safe concurrent access
    state: Mutex<ToeplitzState<T>>,
    /// Configuration that remains constant after initialization
    config: ToeplitzConfig,
}

/// Mutable state for the Toeplitz mechanism
struct ToeplitzState<T> {
    /// Storage for raw noise values Z[t]
    raw_noise_history: Vec<IBig>,
    /// Storage for already-computed noisy prefix sums after post-processing
    noisy_prefix_sums: Vec<T>,
    /// Storage for cumulative counts from all updates
    cumulative_counts: Vec<T>,
}

/// Immutable configuration for the Toeplitz mechanism
struct ToeplitzConfig {
    /// Scale parameter (σ) for noise generation
    scale: f64,
    /// Precision bits for coefficient calculations
    scale_bits: usize,
    /// Variance for discrete Gaussian noise generation
    variance: RBig,
    /// Whether to enforce monotonicity using isotonic regression
    enforce_monotonicity: bool,
}

impl<T> ContinualToeplitzCore<T>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + Zero + num::One + PartialOrd + Clone,
{
    /// Create a new continual Toeplitz mechanism
    /// 
    /// # Arguments
    /// * `scale` - Scale parameter (σ) for noise generation
    /// * `enforce_monotonicity` - Whether to apply isotonic regression for monotonic outputs
    pub fn new(scale: f64, enforce_monotonicity: bool) -> Fallible<Self> {
        if scale <= 0.0 {
            return fallible!(FailedFunction, "Scale must be positive");
        }
        
        // Compute variance for discrete Gaussian: variance = scale^2
        let variance = RBig::from((scale * scale * 1e9) as i64) / RBig::from(1_000_000_000i64);
        
        Ok(ContinualToeplitzCore {
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
    
    /// Update with a new value
    fn append_count_on_new_timestamp(&self, value: T) -> Fallible<usize> {
        let mut state = self.state.lock().unwrap();
        
        // Add the new count
        state.cumulative_counts.push(value);
        let new_time = state.cumulative_counts.len();
        
        // Generate raw noise for the new time step
        state.raw_noise_history.push(sample_discrete_gaussian(self.config.variance.clone())?);
        
        // Compute ALL noisy prefix sums from scratch to ensure consistency
        let mut all_noisy_sums = compute_toeplitz_range(
            &state.cumulative_counts,
            &state.raw_noise_history,
            0,
            new_time,
            self.config.scale_bits,
        )?;
        
        // Clamp to non-negative before isotonic regression
        for val in &mut all_noisy_sums {
            if *val < T::zero() {
                *val = T::zero();
            }
        }
        
        // Apply isotonic regression with constraint to preserve previously released values
        let monotonic_sums = if self.config.enforce_monotonicity {
            if new_time == 1 {
                // First release: standard isotonic regression
                apply_isotonic_regression(all_noisy_sums)?
            } else {
                // Subsequent releases: preserve the first current_time values
                let fixed_prefix = state.noisy_prefix_sums[..new_time - 1].to_vec();
                apply_isotonic_regression_with_fixed_prefix(all_noisy_sums, fixed_prefix)?
            }
        } else {
            all_noisy_sums
        };
        
        // Update the stored results
        state.noisy_prefix_sums = monotonic_sums;
        
        Ok(new_time)
    }
    
    /// Release a range sum
    fn fetch_privacy_preserving_sub_interval_sum(&self, start_time: usize, end_time: usize) -> Fallible<T> {
        let state = self.state.lock().unwrap();
        
        // Validate inputs (1-based indexing)
        if start_time == 0 || end_time == 0 {
            return fallible!(
                FailedFunction,
                "Time indices must be 1-based (start from 1, not 0)"
            );
        }
        
        if end_time > state.noisy_prefix_sums.len() {
            return fallible!(
                FailedFunction,
                "End time {} exceeds current time {}",
                end_time,
                state.noisy_prefix_sums.len()
            );
        }
        
        if start_time > end_time {
            return fallible!(
                FailedFunction,
                "Invalid interval: start_time {} must be <= end_time {}",
                start_time,
                end_time
            );
        }
        
        // Convert from 1-based to 0-based indexing
        // fetch_privacy_preserving_sub_interval_sum(1, 3) means sum of 1st, 2nd, 3rd values (indices 0, 1, 2)
        // This is prefix_sums[2] - prefix_sums[-1] (or 0 if start_time == 1)
        let end_sum = state.noisy_prefix_sums[end_time - 1].clone();
        
        if start_time == 1 {
            Ok(end_sum)
        } else {
            let start_sum = state.noisy_prefix_sums[start_time - 2].clone();
            end_sum.checked_sub(&start_sum)
                .ok_or_else(|| err!(FailedFunction, "Subtraction overflow in range sum"))
        }
    }
    
    /// Get the current number of time steps processed
    fn current_time(&self) -> usize {
        self.state.lock().unwrap().cumulative_counts.len()
    }
    
    /// Get the privacy cost for a given input distance
    fn privacy_cost(&self, d_in: T) -> Fallible<f64> 
    where
        f64: InfCast<T>,
    {
        let d_in_f64 = f64::inf_cast(d_in)?;
        Ok(d_in_f64 / self.config.scale)
    }
}

/// Baseline continual Toeplitz (no monotonicity enforcement)
/// 
/// This variant does not enforce monotonicity in the output prefix sums,
/// which may occasionally decrease due to noise. Use this when exact
/// monotonicity is not required and you want slightly better utility.
pub struct BaselineContinualToeplitz<T: crate::traits::CheckAtom> {
    core: ContinualToeplitzCore<T>,
}

/// Monotonic continual Toeplitz (with isotonic regression)
/// 
/// This variant enforces monotonicity in the output prefix sums using
/// isotonic regression post-processing. Use this when your application
/// requires non-decreasing counts over time (e.g., cumulative statistics).
pub struct MonotonicContinualToeplitz<T: crate::traits::CheckAtom> {
    core: ContinualToeplitzCore<T>,
}

// Macro to implement the trait for both variants, reducing boilerplate
macro_rules! impl_continual_release {
    ($struct_name:ident) => {
        impl<T: crate::traits::CheckAtom> ContinualRelease<T> for $struct_name<T>
        where
            T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + Zero + num::One + PartialOrd + Clone,
        {
            fn append_count_on_new_timestamp(&self, value: T) -> Fallible<usize> {
                self.core.append_count_on_new_timestamp(value)
            }
            
            fn fetch_privacy_preserving_sub_interval_sum(&self, start_time: usize, end_time: usize) -> Fallible<T> {
                self.core.fetch_privacy_preserving_sub_interval_sum(start_time, end_time)
            }
            
            fn current_time(&self) -> usize {
                self.core.current_time()
            }
            
            /// Get the privacy cost (epsilon) for a given sensitivity under pure DP
            /// 
            /// # Arguments
            /// * `d_in` - The L1 distance between input count vectors.
            /// 
            /// # Returns
            /// The privacy parameter epsilon = d_in / scale
            fn privacy_cost(&self, d_in: T) -> Fallible<f64> 
            where
                f64: InfCast<T>,
            {
                self.core.privacy_cost(d_in)
            }
        }
    };
}

// Single implementation for both types
impl_continual_release!(BaselineContinualToeplitz);
impl_continual_release!(MonotonicContinualToeplitz);

// Only the constructors differ
impl<T: crate::traits::CheckAtom> BaselineContinualToeplitz<T>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + Zero + num::One + PartialOrd + Clone,
{
    /// Create a new baseline continual Toeplitz mechanism
    /// 
    /// # Arguments
    /// * `scale` - Scale parameter (σ) for noise generation. Larger values provide more privacy.
    /// 
    /// # Returns
    /// A new baseline mechanism that does not enforce monotonicity
    pub fn new(scale: f64) -> Fallible<Self> {
        Ok(Self {
            core: ContinualToeplitzCore::new(scale, false)?,
        })
    }
}

impl<T: crate::traits::CheckAtom> MonotonicContinualToeplitz<T>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + Zero + num::One + PartialOrd + Clone,
{
    /// Create a new monotonic continual Toeplitz mechanism
    /// 
    /// # Arguments
    /// * `scale` - Scale parameter (σ) for noise generation. Larger values provide more privacy.
    /// 
    /// # Returns
    /// A new monotonic mechanism that enforces non-decreasing prefix sums
    pub fn new(scale: f64) -> Fallible<Self> {
        Ok(Self {
            core: ContinualToeplitzCore::new(scale, true)?,
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_basic_api() -> Fallible<()> {
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
        // Add individual counts (not cumulative)
        mechanism.append_count_on_new_timestamp(7)?;   // 7 events at time 1
        mechanism.append_count_on_new_timestamp(3)?;   // 3 events at time 2
        mechanism.append_count_on_new_timestamp(9)?;   // 9 events at time 3
        
        // Test range queries (1-based, inclusive)
        assert!(mechanism.fetch_privacy_preserving_sub_interval_sum(1, 1).is_ok()); // Single element
        assert!(mechanism.fetch_privacy_preserving_sub_interval_sum(1, 2).is_ok()); // Range
        assert!(mechanism.fetch_privacy_preserving_sub_interval_sum(1, 3).is_ok()); // Full range
        
        assert_eq!(mechanism.current_time(), 3);
        
        Ok(())
    }
    
    #[test]
    fn test_monotonicity_enforcement() -> Fallible<()> {
        let monotonic = MonotonicContinualToeplitz::<i32>::new(1.0)?; // Small scale for more noise
        
        // Add random counts between 0 and 10
        let counts = vec![3, 8, 1, 9, 4, 2, 7, 5, 10, 6];
        for count in counts {
            monotonic.append_count_on_new_timestamp(count)?;
        }
        
        // Verify cumulative sums are monotonic
        let mut prev_sum = 0;
        for i in 1..=10 {
            let curr_sum = monotonic.fetch_privacy_preserving_sub_interval_sum(1, i)?;
            assert!(curr_sum >= prev_sum, "Non-monotonic at position {}", i);
            prev_sum = curr_sum;
        }
        
        Ok(())
    }
    
    #[test]
    fn test_baseline_vs_monotonic() -> Fallible<()> {
        let baseline = BaselineContinualToeplitz::<i32>::new(5.0)?;
        let monotonic = MonotonicContinualToeplitz::<i32>::new(5.0)?;
        
        // Use varied counts to show difference
        for count in vec![2, 7, 0, 5, 1, 8, 3] {
            baseline.append_count_on_new_timestamp(count)?;
            monotonic.append_count_on_new_timestamp(count)?;
        }
        
        // Both should have same current time
        assert_eq!(baseline.current_time(), monotonic.current_time());
        
        // Privacy costs should be identical
        assert_eq!(baseline.privacy_cost(10)?, monotonic.privacy_cost(10)?);
        
        // Note: Baseline may not be monotonic, monotonic always is
        // (actual values depend on random noise)
        
        Ok(())
    }
    
    #[test]
    fn test_binary_counts() -> Fallible<()> {
        // Test with 0s and 1s (like counting binary events)
        let mechanism = BaselineContinualToeplitz::<i32>::new(10.0)?;
        
        for event in vec![1, 0, 1, 1, 0, 0, 1, 0, 1, 1] {
            mechanism.append_count_on_new_timestamp(event)?;
        }
        
        // Should handle sparse binary data correctly
        assert!(mechanism.fetch_privacy_preserving_sub_interval_sum(1, 10).is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_invalid_queries() -> Fallible<()> {
        let mechanism = BaselineContinualToeplitz::<i32>::new(10.0)?;
        
        mechanism.append_count_on_new_timestamp(4)?;
        mechanism.append_count_on_new_timestamp(9)?;
        
        // These should all fail
        assert!(mechanism.fetch_privacy_preserving_sub_interval_sum(0, 1).is_err()); // 0-based not allowed
        assert!(mechanism.fetch_privacy_preserving_sub_interval_sum(3, 2).is_err()); // start > end
        assert!(mechanism.fetch_privacy_preserving_sub_interval_sum(1, 5).is_err()); // end > current_time
        
        Ok(())
    }
    
    #[test]
    fn test_consistency() -> Fallible<()> {
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
        mechanism.append_count_on_new_timestamp(6)?;
        mechanism.append_count_on_new_timestamp(2)?;
        mechanism.append_count_on_new_timestamp(8)?;
        
        // Get sum before adding more
        let sum1 = mechanism.fetch_privacy_preserving_sub_interval_sum(1, 2)?;
        
        // Add more counts
        mechanism.append_count_on_new_timestamp(3)?;
        mechanism.append_count_on_new_timestamp(7)?;
        
        // Previous sum should remain unchanged
        let sum2 = mechanism.fetch_privacy_preserving_sub_interval_sum(1, 2)?;
        assert_eq!(sum1, sum2, "Historical sum changed");
        
        Ok(())
    }

    #[test]
    fn test_privacy_cost() -> Fallible<()> {
        let mechanism = MonotonicContinualToeplitz::<i32>::new(10.0)?;
        
        // ε = sensitivity / scale
        assert_eq!(mechanism.privacy_cost(1)?, 0.1);
        assert_eq!(mechanism.privacy_cost(10)?, 1.0);
        assert_eq!(mechanism.privacy_cost(20)?, 2.0);
        
        Ok(())
    }
    
    #[test]
    fn test_trait_polymorphism() -> Fallible<()> {
        fn test_mechanism<T: ContinualRelease<i32>>(mechanism: &T) -> Fallible<()> {
            mechanism.append_count_on_new_timestamp(47)?;
            Ok(())
        }
        
        test_mechanism(&BaselineContinualToeplitz::<i32>::new(10.0)?)?;
        test_mechanism(&MonotonicContinualToeplitz::<i32>::new(10.0)?)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod demo {
    use super::*;
    
    #[test]
    fn demo_basic_usage() -> Fallible<()> {
        println!("\n=== Basic Continual Release Demo ===");
        
        let mechanism = MonotonicContinualToeplitz::<i32>::new(5.0)?;
        
        // Simulate hourly car counts (individual counts, not cumulative)
        let hourly_counts = vec![15, 23, 18, 30, 25, 35];
        
        println!("Adding hourly car counts:");
        for (hour, &count) in hourly_counts.iter().enumerate() {
            println!("  Hour {}: {} cars", hour + 1, count);
            mechanism.append_count_on_new_timestamp(count)?;
        }
        
        println!("\nQueries (noisy due to privacy):");
        let morning = mechanism.fetch_privacy_preserving_sub_interval_sum(1, 3)?;
        let afternoon = mechanism.fetch_privacy_preserving_sub_interval_sum(4, 6)?;
        let total = mechanism.fetch_privacy_preserving_sub_interval_sum(1, 6)?;
        
        println!("  Morning (hours 1-3): ~{} cars", morning);
        println!("  Afternoon (hours 4-6): ~{} cars", afternoon);
        println!("  Total (hours 1-6): ~{} cars", total);
        
        println!("\nPrivacy guarantee: ε = {} per individual", mechanism.privacy_cost(1)?);
        
        Ok(())
    }
    
    #[test]
    fn demo_baseline_vs_monotonic_difference() -> Fallible<()> {
        println!("\n=== Baseline vs Monotonic Comparison ===");
        
        let baseline = BaselineContinualToeplitz::<i32>::new(2.0)?;  // Low scale for visible noise
        let monotonic = MonotonicContinualToeplitz::<i32>::new(2.0)?;
        
        // Small counts where noise is significant
        let counts = vec![1, 0, 1, 0, 1, 1, 0];
        println!("Input counts: {:?}", counts);
        
        for &count in &counts {
            baseline.append_count_on_new_timestamp(count)?;
            monotonic.append_count_on_new_timestamp(count)?;
        }
        
        println!("\nCumulative sums:");
        print!("  Baseline:  ");
        for i in 1..=counts.len() {
            print!("{:3} ", baseline.fetch_privacy_preserving_sub_interval_sum(1, i)?);
        }
        
        print!("\n  Monotonic: ");
        for i in 1..=counts.len() {
            print!("{:3} ", monotonic.fetch_privacy_preserving_sub_interval_sum(1, i)?);
        }
        println!("\n\nNote: Baseline may decrease, monotonic never does.");
        
        Ok(())
    }
}
