use super::*;

#[test]
fn test_make_bounded_float_checked_sum() -> Fallible<()> {
    let trans = make_bounded_float_checked_sum::<Sequential<f64>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    let trans = make_bounded_float_checked_sum::<Pairwise<f32>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    assert!(make_bounded_float_checked_sum::<Pairwise<f32>>(100000000, (1e20, 1e30)).is_err());

    Ok(())
}

#[test]
fn test_make_sized_bounded_float_checked_sum() -> Fallible<()> {
    let trans = make_sized_bounded_float_checked_sum::<Sequential<f64>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    let trans = make_sized_bounded_float_checked_sum::<Pairwise<f32>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    assert!(
        make_sized_bounded_float_checked_sum::<Pairwise<f32>>(100000000, (1e20, 1e30)).is_err()
    );

    Ok(())
}

#[test]
fn test_round_up_to_nearest_power_of_two() -> Fallible<()> {
    assert_eq!(round_up_to_nearest_power_of_two(1.2)?, 2.);
    assert_eq!(round_up_to_nearest_power_of_two(2.0)?, 2.);
    assert_eq!(round_up_to_nearest_power_of_two(2.1)?, 4.);
    assert_eq!(
        round_up_to_nearest_power_of_two(1e23)?,
        151115727451828646838272.
    );
    assert_eq!(round_up_to_nearest_power_of_two(1e130)?, 11090678776483259438313656736572334813745748301503266300681918322458485231222502492159897624416558312389564843845614287315896631296.);

    Ok(())
}

#[test]
fn test_float_sum_overflows_sequential() -> Fallible<()> {
    let almost_max = f64::from_bits(f64::MAX.to_bits() - 1);
    let ulp_max = f64::MAX - almost_max;
    let largest_size = usize::MAX;

    // should barely fail first check and significantly fail second check
    let can_of = Sequential::<f64>::can_float_sum_overflow(largest_size, (0., ulp_max / 2.))?;
    assert!(can_of);

    // should barely pass first check
    let can_of = Sequential::<f64>::can_float_sum_overflow(largest_size, (0., ulp_max / 4.))?;
    assert!(!can_of);

    // should barely fail first check and significantly pass second check
    let can_of = Sequential::<f64>::can_float_sum_overflow(10, (0., ulp_max / 2.))?;
    assert!(!can_of);
    Ok(())
}

#[test]
fn test_float_sum_overflows_pairwise() -> Fallible<()> {
    let almost_max = f64::from_bits(f64::MAX.to_bits() - 1);
    let ulp_max = f64::MAX - almost_max;
    let largest_size = usize::MAX;

    // should fail both checks
    let can_of = Pairwise::<f64>::can_float_sum_overflow(largest_size, (0., ulp_max / 2.))?;
    assert!(can_of);

    // should barely fail first check and pass second check
    let can_of = Pairwise::<f64>::can_float_sum_overflow(
        largest_size,
        (0., ulp_max / (largest_size as f64)),
    )?;
    assert!(!can_of);

    // should barely pass first check
    let can_of = Pairwise::<f64>::can_float_sum_overflow(
        largest_size,
        (0., ulp_max / (largest_size as f64) / 2.),
    )?;
    assert!(!can_of);

    // should barely fail first check and significantly pass second check
    let can_of =
        Pairwise::<f64>::can_float_sum_overflow(10, (0., ulp_max / (largest_size as f64)))?;
    assert!(!can_of);
    Ok(())
}
