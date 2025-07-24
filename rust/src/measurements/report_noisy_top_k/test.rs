use crate::error::Fallible;

use super::*;

#[test]
fn test_rnm_gumbel() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::new(true);
    let de = make_report_noisy_top_k(input_domain, input_metric, RangeDivergence, 1, 1., false)?;
    let release = de.invoke(&vec![1., 2., 30., 2., 1.])?;
    assert_eq!(release, vec![2]);
    assert_eq!(de.map(&1.0)?, 1.0);

    Ok(())
}

#[test]
fn test_rnm_exponential() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_top_k(input_domain, input_metric, MaxDivergence, 1, 1., false)?;
    let release = de.invoke(&vec![1., 2., 30., 2., 1.])?;
    assert_eq!(release, vec![2]);
    assert_eq!(de.map(&1.0)?, 2.0);

    Ok(())
}

fn check_rnm_outcome<M: SelectionMeasure>(
    measure: M,
    scale: f64,
    negate: bool,
    input: Vec<i32>,
    expected: Vec<usize>,
) -> Fallible<()> {
    let m_rnm = make_report_noisy_top_k(
        VectorDomain::new(AtomDomain::new_non_nan()),
        LInfDistance::default(),
        measure,
        expected.len(),
        scale,
        negate,
    )?;
    assert_eq!(m_rnm.invoke(&input)?, expected);
    Ok(())
}

#[test]
fn test_max_vs_min_gumbel() -> Fallible<()> {
    check_rnm_outcome(RangeDivergence, 0., false, vec![1, 2, 3], vec![2])?;
    check_rnm_outcome(RangeDivergence, 0., true, vec![1, 2, 3], vec![0])?;
    check_rnm_outcome(RangeDivergence, 1., false, vec![1, 1, 100_000], vec![2])?;
    check_rnm_outcome(
        RangeDivergence,
        1.,
        true,
        vec![1, 100_000, 100_000],
        vec![0],
    )?;
    Ok(())
}

#[test]
fn test_max_vs_min_exponential() -> Fallible<()> {
    check_rnm_outcome(MaxDivergence, 0., false, vec![1, 2, 3], vec![2])?;
    check_rnm_outcome(MaxDivergence, 0., true, vec![1, 2, 3], vec![0])?;
    check_rnm_outcome(MaxDivergence, 1., false, vec![1, 1, 100_000], vec![2])?;
    check_rnm_outcome(MaxDivergence, 1., true, vec![1, 100_000, 100_000], vec![0])?;
    Ok(())
}
