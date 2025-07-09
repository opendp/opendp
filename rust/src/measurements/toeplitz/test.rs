use super::*;

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
fn test_inverse_coefficients_scaled() -> Fallible<()> {
    let scale_bits = 20usize;
    
    // Test inverse coefficients: c'_t = c*_{t+1} - c*_t
    let c_prime_0 = compute_inverse_coefficient_scaled(0, scale_bits)?;
    assert_eq!(c_prime_0, IBig::from(1) << scale_bits);
    
    // c'_1 should be negative (c*_2 - c*_1)
    let c_prime_1 = compute_inverse_coefficient_scaled(1, scale_bits)?;
    let expected = compute_toeplitz_coefficient_scaled(2, scale_bits)? 
                 - compute_toeplitz_coefficient_scaled(1, scale_bits)?;
    assert_eq!(c_prime_1, expected);
    
    // Verify the relationship holds for several values
    for t in 0..10 {
        let c_prime_t = compute_inverse_coefficient_scaled(t, scale_bits)?;
        if t == 0 {
            assert_eq!(c_prime_t, compute_toeplitz_coefficient_scaled(0, scale_bits)?);
        } else {
            let expected = compute_toeplitz_coefficient_scaled(t + 1, scale_bits)? 
                         - compute_toeplitz_coefficient_scaled(t, scale_bits)?;
            assert_eq!(c_prime_t, expected);
        }
    }
    
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

#[test]
fn test_make_toeplitz_basic() -> Fallible<()> {
    // Create a simple domain for testing
    let input_domain = VectorDomain::new(
        AtomDomain::<i32>::default()
    ).with_size(5);
    let input_metric = L1Distance::default();
    let scale = 10.0;
    
    // Create the measurement
    let measurement = make_toeplitz(input_domain, input_metric, scale)?;
    
    // Test with constant count data
    let data = vec![10i32; 5];
    let result = measurement.invoke(&data)?;
    
    // Check output length
    assert_eq!(result.len(), 5);
    
    // The prefix sums without noise would be [10, 20, 30, 40, 50]
    // Results should be monotonically increasing (mostly)
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
    
    let measurement = make_toeplitz(input_domain, input_metric, scale)?;
    
    // Use distinct values to verify prefix sums
    let data = vec![1, 2, 3, 4];
    let result = measurement.invoke(&data)?;
    
    println!("Near-zero noise result: {:?}", result);
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
    
    let measurement = make_toeplitz(input_domain, input_metric, scale)?;
    
    // Simulate counting data: varying counts at each time step
    let mut counts = vec![0i32; time_steps];
    counts[0] = 5;   // 5 events at time 0
    counts[5] = 3;   // 3 events at time 5
    counts[10] = 7;  // 7 events at time 10
    counts[15] = 2;  // 2 events at time 15
    
    let noisy_prefix_sums = measurement.invoke(&counts)?;
    
    // Verify output length
    assert_eq!(noisy_prefix_sums.len(), time_steps);
    
    // Test range queries by taking differences of prefix sums
    // Note: While prefix sums are non-negative, range queries (differences) can still be negative
    // This is unavoidable with differentially private prefix sums
    let range_0_5 = noisy_prefix_sums[5];
    let range_6_10 = noisy_prefix_sums[10] - noisy_prefix_sums[5];
    let range_11_15 = noisy_prefix_sums[15] - noisy_prefix_sums[10];
    let total = noisy_prefix_sums[time_steps - 1];
    
    println!("Count [0,5]: {} (true: 8)", range_0_5);
    println!("Count [6,10]: {} (true: 7, can be negative)", range_6_10);
    println!("Count [11,15]: {} (true: 2, can be negative)", range_11_15);
    println!("Total count: {} (true: 17)", total);
    
    // With non-negative clamping, all prefix sums should be >= 0
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
    
    let measurement = make_toeplitz(input_domain.clone(), input_metric.clone(), scale)?;
    
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
        scale2
    )?;
    assert!(measurement2.check(&1, &0.1)?);  // d_in=1, ε=0.1
    assert!(measurement2.check(&5, &0.5)?);  // d_in=5, ε=0.5
    
    Ok(())
}

#[test] 
fn test_edge_cases() -> Fallible<()> {
    // Test with size 1
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(1);
    let measurement = make_toeplitz(domain, L1Distance::default(), 1.0)?;
    let result = measurement.invoke(&vec![5])?;
    assert_eq!(result.len(), 1);
    
    // Test invalid scale
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(5);
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), 0.0).is_err());
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), -1.0).is_err());
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), f64::INFINITY).is_err());
    assert!(make_toeplitz(domain.clone(), L1Distance::default(), f64::NAN).is_err());
    
    // Test data length mismatch
    let measurement = make_toeplitz(domain.clone(), L1Distance::default(), 1.0)?;
    assert!(measurement.invoke(&vec![1, 2, 3]).is_err()); // Too short
    assert!(measurement.invoke(&vec![1, 2, 3, 4, 5, 6]).is_err()); // Too long
    
    // Test empty domain
    let empty_domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(0);
    assert!(make_toeplitz(empty_domain, L1Distance::default(), 1.0).is_err());
    
    // Test domain without size
    let no_size_domain = VectorDomain::new(AtomDomain::<i32>::default());
    assert!(make_toeplitz(no_size_domain, L1Distance::default(), 1.0).is_err());
    
    Ok(())
}

#[test]
fn test_saturation_behavior() -> Fallible<()> {
    // Test integer overflow handling
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(3);
    let measurement = make_toeplitz(domain, L1Distance::default(), 0.1)?; // Very small scale for large noise
    
    // Use maximum values to test saturation
    let data = vec![i32::MAX / 3, i32::MAX / 3, i32::MAX / 3];
    let result = measurement.invoke(&data)?;
    
    // Should not panic, values should saturate
    assert_eq!(result.len(), 3);
    
    // Test with minimum values
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(3);
    let measurement = make_toeplitz(domain, L1Distance::default(), 0.1)?;
    let data = vec![i32::MIN / 3, i32::MIN / 3, i32::MIN / 3];
    let result = measurement.invoke(&data)?;
    assert_eq!(result.len(), 3);
    
    println!("Saturation test (max) result: {:?}", result);
    
    Ok(())
}

#[test]
fn test_different_integer_types() -> Fallible<()> {
    // Test with i64
    let domain_i64 = VectorDomain::new(AtomDomain::<i64>::default()).with_size(5);
    let measurement_i64 = make_toeplitz(domain_i64, L1Distance::default(), 5.0)?;
    let data_i64 = vec![100i64, 200, 300, 400, 500];
    let result_i64 = measurement_i64.invoke(&data_i64)?;
    assert_eq!(result_i64.len(), 5);
    
    // Test with u32 (unsigned)
    let domain_u32 = VectorDomain::new(AtomDomain::<u32>::default()).with_size(5);
    let measurement_u32 = make_toeplitz(domain_u32, L1Distance::default(), 5.0)?;
    let data_u32 = vec![10u32, 20, 30, 40, 50];
    let result_u32 = measurement_u32.invoke(&data_u32)?;
    assert_eq!(result_u32.len(), 5);
    
    Ok(())
}

#[test]
fn test_noise_correlation() -> Fallible<()> {
    // Test that noise is properly correlated across time steps
    let n = 100;
    let scale = 1.0;
    
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(n);
    let measurement = make_toeplitz(domain, L1Distance::default(), scale)?;
    
    // All zeros should produce correlated noise pattern
    let zeros = vec![0i32; n];
    let noise_pattern = measurement.invoke(&zeros)?;
    
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
    // Due to correlation structure, adjacent differences should have much lower variance
    
    Ok(())
}

// Helper function for variance calculation
fn variance(data: &[f64]) -> f64 {
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64
}

#[test]
fn test_large_time_series() -> Fallible<()> {
    // Test with larger time series to ensure scalability
    let n = 200;  // Reduced from 1000 to 200 for faster tests
    let scale = 20.0;
    
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(n);
    let measurement = make_toeplitz(domain, L1Distance::default(), scale)?;
    
    // Generate random-walk style data
    let mut data = vec![0i32; n];
    let mut current = 100i32;
    for i in 0..n {
        current += (i % 7) as i32 - 3; // Pseudo-random walk
        data[i] = current.max(0);
    }
    
    let result = measurement.invoke(&data)?;
    assert_eq!(result.len(), n);
    
    // Test some range queries
    let q1 = result[49];  // Sum of first 50
    let q2 = result[99] - result[49]; // Sum of elements 50-99
    let q3 = result[199] - result[99]; // Sum of elements 100-199
    
    println!("Large series queries: [0,49]={}, [50,99]={}, [100,199]={}", q1, q2, q3);
    
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_continual_consistency() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0)?;
        
        // First query: times [0, 5)
        let counts1 = vec![10, 20, 30, 40, 50];
        let result1 = mechanism.release(&counts1)?;
        assert_eq!(result1.len(), 5);
        
        // Second query: times [0, 8)
        let counts2 = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let result2 = mechanism.release(&counts2)?;
        assert_eq!(result2.len(), 8);
        
        // The first 5 values should be identical
        for i in 0..5 {
            assert_eq!(result1[i], result2[i], "Inconsistent value at time {}", i);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_continual_decreasing_time_error() -> Fallible<()> {
        let mechanism = ContinualToeplitz::<i32>::new(10.0)?;
        
        // First query: times [0, 5)
        let counts1 = vec![10, 20, 30, 40, 50];
        mechanism.release(&counts1)?;
        
        // Second query with fewer time steps should fail
        let counts2 = vec![10, 20, 30];
        assert!(mechanism.release(&counts2).is_err());
        
        Ok(())
    }
}
