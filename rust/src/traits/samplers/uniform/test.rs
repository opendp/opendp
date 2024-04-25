use super::*;
use std::collections::HashMap;

#[test]
fn test_uniform_int_below() -> Fallible<()> {
    assert!(u32::sample_uniform_int_below(7, Some(0)).is_err());

    let sample = u32::sample_uniform_int_below(7, None)?;
    assert!(sample < 7);

    // odds of failing this test are 1 in 1/64^1000
    let sample = u32::sample_uniform_int_below(7, Some(1000))?;
    assert!(sample < 7);
    Ok(())
}

#[test]
#[ignore]
fn test_sample_uniform_int_below() -> Fallible<()> {
    let mut counts = HashMap::new();
    // this checks that the output distribution of each number is uniform
    (0..10000).try_for_each(|_| {
        let sample = u32::sample_uniform_int_below(7, None)?;
        *counts.entry(sample).or_insert(0) += 1;
        Fallible::Ok(())
    })?;
    println!("{:?}", counts);
    Ok(())
}

#[test]
#[ignore]
fn test_sample_uniform_int_below_ubig() -> Fallible<()> {
    let mut counts = HashMap::new();
    // this checks that the output distribution of each number is uniform
    (0..10000).try_for_each(|_| {
        let sample = UBig::sample_uniform_int_below(UBig::from(255u8), None)?;
        *counts.entry(sample).or_insert(0) += 1;
        Fallible::Ok(())
    })?;
    println!("{:?}", counts);
    Ok(())
}

#[test]
#[ignore]
fn test_sample_uniform_int() -> Fallible<()> {
    let mut counts = HashMap::new();
    // this checks that the output distribution of each number is uniform
    (0..10000).try_for_each(|_| {
        let sample = u32::sample_uniform_int()?;
        *counts.entry(sample).or_insert(0) += 1;
        Fallible::Ok(())
    })?;
    println!("{:?}", counts);
    Ok(())
}
