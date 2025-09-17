use std::array::from_fn;

use crate::traits::samplers::test::check_chi_square;

use super::*;

#[test]
fn test_rnm_gumbel_distribution_varied() -> Fallible<()> {
    let scores: [_; 10] = from_fn(|i| i);
    let trials = 1000;
    let mut observed = [0.0; 10];
    (0..trials).try_for_each(|_| {
        observed[gumbel_top_k(&scores, 1.0, 1, false)?[0]] += 1.0;
        Fallible::Ok(())
    })?;

    // compute softmax to get expected
    let numer: f64 = (0..10).map(|i| (i as f64).exp()).sum();
    let expected = from_fn(|i| (i as f64).exp() / numer * (trials as f64));

    check_chi_square(observed, expected)
}

#[test]
fn test_top() -> Fallible<()> {
    // Basic test cases
    let res = top_k(vec![1, 2, 3].into_iter(), 2, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![3, 2]);

    // Test empty input
    let res: Vec<i32> = top_k(vec![].into_iter(), 2, |a, b| Ok(a > b))?;
    assert_eq!(res, Vec::<i32>::new());

    // Test k=0
    let res: Vec<i32> = top_k(vec![1, 2, 3].into_iter(), 0, |a, b| Ok(a > b))?;
    assert_eq!(res, Vec::<i32>::new());

    // Test k larger than input
    let res = top_k(vec![1, 2].into_iter(), 3, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![2, 1]);

    // Test with duplicates
    let res = top_k(vec![3, 2, 3, 1, 3].into_iter(), 2, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![3, 3]);

    // Test with negative numbers
    let res = top_k(vec![-1, -2, -3].into_iter(), 1, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![-1]);

    // Test min instead of max
    let res = top_k(vec![1, 2, 3].into_iter(), 2, |a, b| Ok(a < b))?;
    assert_eq!(res, vec![1, 2]);

    // Terminates when equal
    let res = top_k(vec![1, 2, 2].into_iter(), 2, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![2, 2]);

    Ok(())
}

#[test]
fn test_partition_point_mut() -> Fallible<()> {
    // before
    let out = partition_point_mut(&mut vec![1, 2, 3], |&mut x| Ok(x < -1))?;
    assert_eq!(out, 0);

    // middle
    let out = partition_point_mut(&mut vec![1, 2, 3], |&mut x| Ok(x < 2))?;
    assert_eq!(out, 1);

    // after
    let out = partition_point_mut(&mut vec![1, 2, 3], |&mut x| Ok(x < 5))?;
    assert_eq!(out, 3);

    // after
    let out = partition_point_mut(&mut Vec::<u32>::new(), |&mut x| Ok(x < 2))?;
    assert_eq!(out, 0);

    // error
    let res = partition_point_mut(&mut vec![1, 2, 3], |_| fallible!(FailedFunction));
    assert!(res.is_err());

    Ok(())
}

#[test]
fn test_binary_search_by_mut() -> Fallible<()> {
    // before
    let out = binary_search_by_mut(&mut vec![1, 2, 3], |&mut x| Ok(x.cmp(&-1)))?;
    assert_eq!(out, 0);

    // middle
    let out = binary_search_by_mut(&mut vec![1, 2, 3], |&mut x| Ok(x.cmp(&2)))?;
    assert_eq!(out, 1);

    // after
    let out = binary_search_by_mut(&mut vec![1, 2, 3], |&mut x| Ok(x.cmp(&5)))?;
    assert_eq!(out, 3);

    // empty data
    let out = binary_search_by_mut(&mut Vec::<i32>::new(), |&mut x| Ok(x.cmp(&2)))?;
    assert_eq!(out, 0);

    // error
    let res = binary_search_by_mut(&mut vec![1, 2, 3], |_| fallible!(FailedFunction));
    assert!(res.is_err());

    Ok(())
}
