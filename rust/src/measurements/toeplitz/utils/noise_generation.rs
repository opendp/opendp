use crate::error::*;
use crate::traits::Integer;
use dashu::integer::IBig;
use num::{CheckedAdd, CheckedMul, CheckedSub, Zero};
use std::str::FromStr;

use super::type_conversion::from_ibig_saturating;

/// Apply the inverse Toeplitz transformation to compute correlated noise
/// This implements the summation in Equation (2.3) for a single time step
pub(crate) fn compute_correlated_noise_at_time(
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
pub(crate) fn apply_correlated_noise<T>(
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

/// Compute the approximately optimal Toeplitz coefficient c*_t:
/// c*_t = (-1)^t \cdot {-1/2 \choose t} = 2^{-2t} \cdot {2t \choose t}
/// 
/// This is the t-th coefficient of the matrix C = A_pre^(1/2). Note that a Toeplitz matrix is one where you can describe the entire matrix by its first column.
/// For numerical stability, we scale up by 2^{scale_bits}.
fn compute_toeplitz_coefficient_scaled(t: usize, scale_bits: usize) -> Fallible<IBig> {
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
    
    // The coefficient is: {2t \choose t} / 4^t * 2^{scale_bits}
    // Since 4^t = 2^{2t}, this is: {2t \choose t} * 2^{scale_bits - 2t}
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
fn compute_inverse_coefficient_scaled(t: usize, scale_bits: usize) -> Fallible<IBig> {
    if t == 0 {
        // c'_0 = 1 (scaled by 2^{scale_bits})
        return Ok(IBig::from(1) << scale_bits);
    }
    
    // Using the relation: c'_t = c*_{t+1} - c*_t
    let c_t = compute_toeplitz_coefficient_scaled(t, scale_bits)?;
    let c_t_plus_1 = compute_toeplitz_coefficient_scaled(t + 1, scale_bits)?;
    Ok(c_t_plus_1 - c_t)
}


#[test]
fn test_toeplitz_coefficients_scaled() -> Fallible<()> {
    // Test the mathematical correctness of Toeplitz coefficients
    let scale_bits = 20usize; // Use 2^20 for scaling
    
    // c*_0 = 1 * 2^20
    let c0 = compute_toeplitz_coefficient_scaled(0, scale_bits)?;
    assert_eq!(c0, IBig::from(1) << scale_bits);
    
    // c*_1 = (2 choose 1) / 4^1 * 2^20 = 2/4 * 2^20 = 2^19
    let c1 = compute_toeplitz_coefficient_scaled(1, scale_bits)?;
    assert_eq!(c1, IBig::from(1) << (scale_bits - 1));
    
    // c*_2 = (4 choose 2) / 4^2 * 2^20 = 6/16 * 2^20 = 3/8 * 2^20
    let c2 = compute_toeplitz_coefficient_scaled(2, scale_bits)?;
    assert_eq!(c2, IBig::from(3) << (scale_bits - 3));
    
    // c*_3 = (6 choose 3) / 4^3 * 2^20 = 20/64 * 2^20 = 5/16 * 2^20
    let c3 = compute_toeplitz_coefficient_scaled(3, scale_bits)?;
    assert_eq!(c3, IBig::from(5) << (scale_bits - 4));
    
    // Test that coefficients decrease
    assert!(c0 > c1);
    assert!(c1 > c2);
    assert!(c2 > c3);
    
    Ok(())
}

#[test]
fn test_coefficient_precision() -> Fallible<()> {
    // Test with different scale_bits to ensure precision handling
    for scale_bits in [10, 20, 30, 40, 50] {
        let c0 = compute_toeplitz_coefficient_scaled(0, scale_bits)?;
        assert_eq!(c0, IBig::from(1) << scale_bits);
        
        // Test that we don't lose precision with large scale_bits
        let c5 = compute_toeplitz_coefficient_scaled(5, scale_bits)?;
        assert!(c5 > IBig::zero());
    }
    
    Ok(())
}
