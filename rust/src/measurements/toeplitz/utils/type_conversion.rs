use crate::error::*;
use crate::traits::Integer;
use dashu::integer::IBig;
use num::{CheckedAdd, CheckedMul, CheckedSub, Zero};
use std::fmt::Display;
use std::str::FromStr;

/// Convert an integer type to IBig
pub(crate) fn to_ibig<T: Display>(value: &T) -> Fallible<IBig> {
    IBig::from_str(&value.to_string())
        .map_err(|_| err!(FailedFunction, "failed to convert to IBig"))
}

/// Convert IBig to an integer type T with saturation
pub(crate) fn from_ibig_saturating<T>(ibig: IBig) -> Fallible<T> 
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

/// Helper function to compute prefix sum up to time t
pub(crate) fn compute_prefix_sum<T>(data: &[T], t: usize) -> Fallible<IBig>
where
    T: Display,
{
    let mut sum = IBig::zero();
    for i in 0..=t {
        sum += to_ibig(&data[i])?;
    }
    Ok(sum)
}

#[test]
fn test_ibig_conversion() -> Fallible<()> {
    // Test to_ibig conversion
    assert_eq!(to_ibig(&42i32)?, IBig::from(42));
    assert_eq!(to_ibig(&-100i64)?, IBig::from(-100));
    assert_eq!(to_ibig(&i32::MAX)?, IBig::from(i32::MAX));
    
    // Test from_ibig_saturating
    assert_eq!(from_ibig_saturating::<i32>(IBig::from(42))?, 42);
    assert_eq!(from_ibig_saturating::<i32>(IBig::from(-100))?, -100);
    
    // Test saturation behavior
    let huge = IBig::from(i64::MAX) * IBig::from(2);
    assert_eq!(from_ibig_saturating::<i32>(huge)?, i32::MAX);
    
    let huge_neg = IBig::from(i64::MIN) * IBig::from(2);
    assert_eq!(from_ibig_saturating::<i32>(huge_neg)?, i32::MIN);
    
    Ok(())
}
