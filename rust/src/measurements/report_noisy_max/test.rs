use core::f64;

use crate::error::Fallible;

use super::*;

fn check_rnm_outcome<M: TopKMeasure>(
    measure: M,
    scale: f64,
    optimize: Optimize,
    input: Vec<i32>,
    expected_idx: usize,
    expected_loss: f64,
) -> Fallible<()> {
    let m_rnm = make_noisy_max(
        VectorDomain::new(AtomDomain::default()),
        LInfDistance::default(),
        measure,
        scale,
        optimize,
    )?;
    assert_eq!(m_rnm.invoke(&input)?, expected_idx);
    assert_eq!(m_rnm.map(&1)?, expected_loss);
    Ok(())
}

#[test]
fn test_max_vs_min_gumbel() -> Fallible<()> {
    check_rnm_outcome(
        RangeDivergence,
        0.,
        Optimize::Max,
        vec![1, 2, 3],
        2,
        f64::INFINITY,
    )?;
    check_rnm_outcome(
        RangeDivergence,
        0.,
        Optimize::Min,
        vec![1, 2, 3],
        0,
        f64::INFINITY,
    )?;
    check_rnm_outcome(
        RangeDivergence,
        1.,
        Optimize::Max,
        vec![1, 1, 100_000],
        2,
        2.0,
    )?;
    check_rnm_outcome(
        RangeDivergence,
        1.,
        Optimize::Min,
        vec![1, 100_000, 100_000],
        0,
        2.0,
    )?;
    Ok(())
}

#[test]
fn test_max_vs_min_exponential() -> Fallible<()> {
    check_rnm_outcome(
        MaxDivergence,
        0.,
        Optimize::Max,
        vec![1, 2, 3],
        2,
        f64::INFINITY,
    )?;
    check_rnm_outcome(
        MaxDivergence,
        0.,
        Optimize::Min,
        vec![1, 2, 3],
        0,
        f64::INFINITY,
    )?;
    check_rnm_outcome(
        MaxDivergence,
        1.,
        Optimize::Max,
        vec![1, 1, 100_000],
        2,
        2.0,
    )?;
    check_rnm_outcome(
        MaxDivergence,
        1.,
        Optimize::Min,
        vec![1, 100_000, 100_000],
        0,
        2.0,
    )?;
    Ok(())
}
