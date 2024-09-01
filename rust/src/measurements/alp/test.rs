use super::*;

fn idx<T>(i: usize) -> Arc<dyn Fn(&T) -> usize + Send + Sync> {
    Arc::new(move |_| i)
}

// Functions that always return its index
fn index_identify_functions<T>(n: usize) -> Vec<HashFunction<T>> {
    (0..n).map(|i| idx(i)).collect::<Vec<HashFunction<T>>>()
}

#[test]
fn test_exponent_next_power_of_two() -> Fallible<()> {
    assert_eq!(exponent_next_power_of_two(1 as u64), 0);

    assert_eq!(exponent_next_power_of_two(2 as u64), 1);

    assert_eq!(exponent_next_power_of_two(3 as u64), 2);

    assert_eq!(exponent_next_power_of_two(7 as u64), 3);

    Ok(())
}

#[test]
fn test_hash() -> Fallible<()> {
    assert_eq!(hash(3, 4, 5, 64), 17);
    assert_eq!(hash(3, 4, 5, 63), 8);

    assert_eq!(hash(1, u64::MAX, 0, 2), 3);
    assert_eq!(hash(1, u64::MAX, 0, 3), 7);

    assert_eq!(hash(4, u64::MAX, 0, 16), (1 << 16) - 1);

    Ok(())
}

#[test]
fn test_sample_hash() -> Fallible<()> {
    let h = sample_hash_function(5)?;

    for i in 0u64..20u64 {
        assert!(h(&i) < (1 << 5));
    }

    Ok(())
}

#[test]
fn test_alp_construction() -> Fallible<()> {
    let beta = 10;
    let alp = make_alp_state_with_hashers::<u32, u32>(
        MapDomain::default(),
        L1Distance::default(),
        1.0,
        1.,
        beta,
        index_identify_functions(beta),
    )?;

    assert_eq!(alp.map(&1)?, 1.);

    let mut x = HashMap::new();
    x.insert(42, 10);

    alp.function.eval(&x.clone())?;

    // Values exceeding beta is truncated internally
    x.insert(42, 10000);
    alp.function.eval(&x.clone())?;

    Ok(())
}

#[test]
fn test_alp_construction_out_of_range() -> Fallible<()> {
    let s = 5;
    // Hash functions return values out of range
    // Handle silently using modulo
    // Returning an error would violate privacy
    let h = index_identify_functions(20);
    let alp = make_alp_state_with_hashers::<u32, u32>(
        MapDomain::default(),
        L1Distance::default(),
        1.0,
        1.,
        s,
        h,
    )?;

    let mut x = HashMap::new();
    x.insert(42, 3);

    alp.function.eval(&x.clone())?;

    Ok(())
}

#[test]
fn test_estimate_unary() -> Fallible<()> {
    let z1 = vec![true, true, true, false, true, false, false, true];
    assert!(estimate_unary(&z1) == 4.0);

    let z2 = vec![true, false, false, false, true, false, false, true];
    assert!(estimate_unary(&z2) == 1.0);

    let z3 = vec![false, true, true, false, false, true, false, true];
    assert!(estimate_unary(&z3) == 3.0);

    Ok(())
}

#[test]
fn test_compute_estimate() -> Fallible<()> {
    let z1 = vec![true, true, true, false, true, false, false, true];
    assert!(
        compute_estimate(
            &AlpState {
                alpha: 3.,
                scale: 1.0,
                hashers: index_identify_functions(8),
                z: z1
            },
            &0
        ) == 12.0
    );

    let z2 = vec![true, false, false, false, true, false, false, true];
    assert!(
        compute_estimate(
            &AlpState {
                alpha: 1.,
                scale: 2.0,
                hashers: index_identify_functions(8),
                z: z2
            },
            &0
        ) == 0.5
    );

    let z3 = vec![false, true, true, false, false, true, false, true];
    assert!(
        compute_estimate(
            &AlpState {
                alpha: 1.,
                scale: 0.5,
                hashers: index_identify_functions(8),
                z: z3
            },
            &0
        ) == 6.0
    );

    Ok(())
}

#[test]
fn test_construct_and_post_process() -> Fallible<()> {
    let mut x = HashMap::new();
    x.insert(0, 7);
    x.insert(42, 12);
    x.insert(100, 5);

    let alp_meas = make_alp_state::<i32, i32>(
        MapDomain::default(),
        L1Distance::default(),
        2.,
        24,
        Some(24),
        None,
        None,
    )?;
    let alp_state = alp_meas.invoke(&x)?;

    let postprocessor = post_alp_state_to_queryable();
    let mut queryable = postprocessor.eval(&alp_state)?;

    queryable.eval(&0)?;
    queryable.eval(&42)?;
    queryable.eval(&100)?;
    queryable.eval(&1000)?;

    Ok(())
}

#[test]
fn test_post_process_measurement() -> Fallible<()> {
    let mut x = HashMap::new();
    x.insert(0, 7);
    x.insert(42, 12);
    x.insert(100, 5);

    let alp_meas = make_alp_queryable::<i32, i32>(
        MapDomain {
            key_domain: AtomDomain::default(),
            value_domain: AtomDomain::default(),
        },
        L1Distance::default(),
        2.,
        24,
        Some(24),
        None,
        None,
    )?;

    assert_eq!(alp_meas.map(&1)?, 2.);
    let mut queryable = alp_meas.invoke(&x)?;

    queryable.eval(&0)?;
    queryable.eval(&42)?;
    queryable.eval(&100)?;
    queryable.eval(&1000)?;

    Ok(())
}
