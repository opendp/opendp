use crate::error::*;
use crate::traits::Integer;
use dashu::integer::IBig;
use num::{CheckedAdd, CheckedMul, CheckedSub};
use std::fmt::Display;
use std::str::FromStr;

use super::type_conversion::{to_ibig, from_ibig_saturating};

/// Apply isotonic regression using the Pool Adjacent Violators Algorithm (PAVA)
/// 
/// PAVA runs in O(n) time and ensures that the output is the best MSE-fitting of the input data that respects non-decreasing monotonicity.
/// 
/// The Pool Adjacent Violators Algorithm was introduced by Barlow et al. (1972) and
/// further developed by Robertson et al. (1988). It works by iteratively pooling
/// adjacent values that violate the monotonicity constraint, replacing them with
/// their weighted average.
/// 
/// The post-processing property of differential privacy (Dwork et al., 2006) guarantees
/// that this deterministic transformation preserves the ε-differential privacy of the input.
/// Another way to think about this is: all the computations here can be done deterministically with the
/// noisy counts after the Toeplitz mechanism, through local computations by the adversary,
/// so the two views with or without this isotonic regression step are identical.
pub(crate) fn apply_isotonic_regression<T>(mut values: Vec<T>) -> Fallible<Vec<T>>
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
        let (start1, _end1, sum1, count1) = &blocks[i];
        let (_start2, end2, sum2, count2) = &blocks[i + 1];
        
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

/// Apply isotonic regression while keeping the first k values fixed
/// 
/// This modifies the isotonic regression algorithm to treat the first k values
/// as immutable constraints.
pub(crate) fn apply_isotonic_regression_with_fixed_prefix<T>(
    mut values: Vec<T>,
    fixed_prefix: Vec<T>,
) -> Fallible<Vec<T>>
where
    T: Integer + Display + FromStr + CheckedAdd + CheckedMul + CheckedSub + num::One + PartialOrd + Clone,
{
    let n = values.len();
    let k = fixed_prefix.len();
    
    if k > n {
        return fallible!(FailedFunction, "fixed prefix length {} exceeds total length {}", k, n);
    }
    
    // Replace the first k values with the fixed prefix
    for i in 0..k {
        values[i] = fixed_prefix[i].clone();
    }
    
    // If k == n, we're done
    if k == n {
        return Ok(values);
    }
    
    // Apply isotonic regression starting from position k but ensure
    // values[k] >= values[k-1]
    if k > 0 && values[k] < values[k - 1] {
        values[k] = values[k - 1].clone();
    }
    
    // Now apply isotonic regression only to positions k through n-1
    // We need to ensure monotonicity from position k onwards
    let mut blocks = Vec::new();
    
    // Add the fixed portion as a single immutable block
    if k > 0 {
        let sum = values[..k].iter()
            .try_fold(IBig::from(0), |acc, v| Ok::<IBig, Error>(acc + to_ibig(v)?))?;
        blocks.push((0, k - 1, sum, k, true)); // true = fixed
    }
    
    // Initialize blocks for the non-fixed portion
    for i in k..n {
        blocks.push((i, i, to_ibig(&values[i])?, 1, false));  // false = not fixed
    }
    
    // Pool adjacent violators, but never merge with fixed blocks
    let mut i = if k > 0 { 1 } else { 0 };
    while i < blocks.len().saturating_sub(1) {
        let (start1, _end1, sum1, count1, fixed1) = blocks[i].clone();
        let (_start2, end2, sum2, count2, fixed2) = blocks[i + 1].clone();
        
        // Never merge with fixed blocks
        if fixed1 || fixed2 {
            i += 1;
            continue;
        }
        
        // Check if monotonicity is violated
        let avg1 = &sum1 * IBig::from(count2);
        let avg2 = &sum2 * IBig::from(count1);
        
        if avg1 > avg2 {
            // Pool the blocks
            let new_sum = sum1 + sum2;
            let new_count = count1 + count2;
            blocks[i] = (start1, end2, new_sum, new_count, false);
            blocks.remove(i + 1);
            
            // Check if we need to pool with previous blocks (if not fixed)
            if i > 0 && !blocks[i - 1].4 {
                i -= 1;
            }
        } else {
            i += 1;
        }
    }
    
    // Reconstruct the sequence
    for (start, end, sum, count, fixed) in blocks {
        if !fixed {
            let avg = sum / IBig::from(count);
            let avg_t = from_ibig_saturating::<T>(avg)?;
            
            for j in start..=end {
                values[j] = avg_t.clone();
            }
        }
    }
    
    // Final pass to ensure strict monotonicity
    for i in 1..n {
        if values[i] < values[i - 1] {
            values[i] = values[i - 1].clone();
        }
    }
    
    Ok(values)
}

#[test]
fn test_isotonic_regression_properties() -> Fallible<()> {
    // Test that isotonic regression preserves key properties
    
    // Test 1: Already monotonic sequence remains unchanged
    let monotonic = vec![1, 2, 3, 4, 5];
    let result = apply_isotonic_regression(monotonic.clone())?;
    assert_eq!(result, monotonic);
    
    // Test 2: Simple violation gets corrected
    let violated = vec![1, 3, 2, 4, 5];
    let result = apply_isotonic_regression(violated)?;
    // Should pool 3 and 2 to their average 2.5, rounded to 2 for integers
    assert!(result[1] >= result[0]);
    assert!(result[2] >= result[1]);
    assert!(result[3] >= result[2]);
    assert!(result[4] >= result[3]);
    
    // Test 3: Multiple violations
    let multi_violated = vec![5, 4, 3, 2, 1];
    let result = apply_isotonic_regression(multi_violated)?;
    // Should all be pooled to average 3
    for &val in &result {
        assert_eq!(val, 3);
    }
    
    Ok(())
}
