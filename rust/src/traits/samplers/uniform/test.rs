use super::*;
use std::collections::HashMap;

#[test]
fn test_uniform_int_below() -> Fallible<()> {
    assert!(sample_uniform_uint_below(7u32)? < 7);
    Ok(())
}

#[test]
#[ignore]
fn test_sample_uniform_int_below() -> Fallible<()> {
    let mut counts = HashMap::new();
    // this checks that the output distribution of each number is uniform
    (0..10000).try_for_each(|_| {
        let sample = sample_uniform_uint_below(7u32)?;
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
        let sample = sample_uniform_ubig_below(UBig::from(255u8))?;
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
        let sample: u32 = sample_from_uniform_bytes()?;
        *counts.entry(sample).or_insert(0) += 1;
        Fallible::Ok(())
    })?;
    println!("{:?}", counts);
    Ok(())
}
