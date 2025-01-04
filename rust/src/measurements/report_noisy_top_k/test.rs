use crate::error::Fallible;

use super::*;

#[test]
fn test_rnm_gumbel() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::new(true);
    let de = make_report_noisy_top_k(
        input_domain,
        input_metric,
        RangeDivergence,
        1,
        1.,
        Optimize::Max,
    )?;
    let release = de.invoke(&vec![1., 2., 30., 2., 1.])?;
    assert_eq!(release, vec![2]);
    assert_eq!(de.map(&1.0)?, 1.0);

    Ok(())
}

#[test]
fn test_rnm_exponential() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_top_k(
        input_domain,
        input_metric,
        MaxDivergence,
        1,
        1.,
        Optimize::Max,
    )?;
    let release = de.invoke(&vec![1., 2., 30., 2., 1.])?;
    assert_eq!(release, vec![2]);
    assert_eq!(de.map(&1.0)?, 2.0);

    Ok(())
}

fn check_rnm_outcome<M: SelectionMeasure>(
    measure: M,
    scale: f64,
    optimize: Optimize,
    input: Vec<i32>,
    expected: Vec<usize>,
) -> Fallible<()> {
    let m_rnm = make_report_noisy_top_k(
        VectorDomain::new(AtomDomain::new_non_nan()),
        LInfDistance::default(),
        measure,
        expected.len(),
        scale,
        optimize,
    )?;
    assert_eq!(m_rnm.invoke(&input)?, expected);
    Ok(())
}

#[test]
fn test_max_vs_min_gumbel() -> Fallible<()> {
    check_rnm_outcome(RangeDivergence, 0., Optimize::Max, vec![1, 2, 3], vec![2])?;
    check_rnm_outcome(RangeDivergence, 0., Optimize::Min, vec![1, 2, 3], vec![0])?;
    check_rnm_outcome(
        RangeDivergence,
        1.,
        Optimize::Max,
        vec![1, 1, 100_000],
        vec![2],
    )?;
    check_rnm_outcome(
        RangeDivergence,
        1.,
        Optimize::Min,
        vec![1, 100_000, 100_000],
        vec![0],
    )?;
    Ok(())
}

#[test]
fn test_max_vs_min_exponential() -> Fallible<()> {
    check_rnm_outcome(MaxDivergence, 0., Optimize::Max, vec![1, 2, 3], vec![2])?;
    check_rnm_outcome(MaxDivergence, 0., Optimize::Min, vec![1, 2, 3], vec![0])?;
    check_rnm_outcome(
        MaxDivergence,
        1.,
        Optimize::Max,
        vec![1, 1, 100_000],
        vec![2],
    )?;
    check_rnm_outcome(
        MaxDivergence,
        1.,
        Optimize::Min,
        vec![1, 100_000, 100_000],
        vec![0],
    )?;
    Ok(())
}

#[test]
fn test_top() -> Fallible<()> {
    // Basic test cases
    let res = top(vec![1, 2, 3].into_iter(), 2, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![3, 2]);

    // Test empty input
    let res: Vec<i32> = top(vec![].into_iter(), 2, |a, b| Ok(a > b))?;
    assert_eq!(res, Vec::<i32>::new());

    // Test k=0
    let res: Vec<i32> = top(vec![1, 2, 3].into_iter(), 0, |a, b| Ok(a > b))?;
    assert_eq!(res, Vec::<i32>::new());

    // Test k larger than input
    let res = top(vec![1, 2].into_iter(), 3, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![2, 1]);

    // Test with duplicates
    let res = top(vec![3, 2, 3, 1, 3].into_iter(), 2, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![3, 3]);

    // Test with negative numbers
    let res = top(vec![-1, -2, -3].into_iter(), 1, |a, b| Ok(a > b))?;
    assert_eq!(res, vec![-1]);

    // Test min instead of max
    let res = top(vec![1, 2, 3].into_iter(), 2, |a, b| Ok(a < b))?;
    assert_eq!(res, vec![1, 2]);

    // Terminates when equal
    let res = top(vec![1, 2, 2].into_iter(), 2, |a, b| Ok(a > b))?;
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
