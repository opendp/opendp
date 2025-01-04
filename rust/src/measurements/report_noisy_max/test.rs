use crate::{
    error::Fallible,
    measures::{MaxDivergence, RangeDivergence},
};

use super::*;

#[test]
fn test_rnm_gumbel() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_max(
        input_domain,
        input_metric,
        RangeDivergence,
        1.,
        Optimize::Max,
    )?;
    let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
    println!("{:?}", release);

    Ok(())
}

fn check_rnm_outcome<M: SelectionMeasure>(
    measure: M,
    scale: f64,
    optimize: Optimize,
    input: Vec<i32>,
    expected: usize,
) -> Fallible<()> {
    let m_rnm = make_report_noisy_max(
        VectorDomain::new(AtomDomain::default()),
        LInfDistance::default(),
        measure,
        scale,
        optimize,
    )?;
    assert_eq!(m_rnm.invoke(&input)?, expected);
    Ok(())
}

#[test]
fn test_max_vs_min_gumbel() -> Fallible<()> {
    check_rnm_outcome(RangeDivergence, 0., Optimize::Max, vec![1, 2, 3], 2)?;
    check_rnm_outcome(RangeDivergence, 0., Optimize::Min, vec![1, 2, 3], 0)?;
    check_rnm_outcome(RangeDivergence, 1., Optimize::Max, vec![1, 1, 100_000], 2)?;
    check_rnm_outcome(
        RangeDivergence,
        1.,
        Optimize::Min,
        vec![1, 100_000, 100_000],
        0,
    )?;
    Ok(())
}

#[test]
fn test_rnm_exponential() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_max(input_domain, input_metric, MaxDivergence, 1., Optimize::Max)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
    println!("{:?}", release);

    Ok(())
}

#[test]
fn test_max_vs_min_exponential() -> Fallible<()> {
    check_rnm_outcome(MaxDivergence, 0., Optimize::Max, vec![1, 2, 3], 2)?;
    check_rnm_outcome(MaxDivergence, 0., Optimize::Min, vec![1, 2, 3], 0)?;
    check_rnm_outcome(MaxDivergence, 1., Optimize::Max, vec![1, 1, 100_000], 2)?;
    check_rnm_outcome(
        MaxDivergence,
        1.,
        Optimize::Min,
        vec![1, 100_000, 100_000],
        0,
    )?;
    Ok(())
}
