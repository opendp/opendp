use crate::{
    error::Fallible,
    measures::{MaxDivergence, RangeDivergence},
};

use super::*;

fn check_rnm_outcome<M: SelectionMeasure>(
    measure: M,
    scale: f64,
    negate: bool,
    input: Vec<i32>,
    expected_idx: usize,
    expected_loss: f64,
) -> Fallible<()> {
    let m_rnm = make_report_noisy_max(
        VectorDomain::new(AtomDomain::default()),
        LInfDistance::default(),
        measure,
        scale,
        negate,
    )?;
    assert_eq!(m_rnm.invoke(&input)?, expected_idx);
    assert_eq!(m_rnm.map(&1)?, expected_loss);
    Ok(())
}

#[test]
fn test_max_vs_min_gumbel() -> Fallible<()> {
    check_rnm_outcome(RangeDivergence, 0., false, vec![1, 2, 3], 2, f64::INFINITY)?;
    check_rnm_outcome(RangeDivergence, 0., true, vec![1, 2, 3], 0, f64::INFINITY)?;
    check_rnm_outcome(RangeDivergence, 1., false, vec![1, 1, 100_000], 2, 2.0)?;
    check_rnm_outcome(RangeDivergence, 1., true, vec![1, 100_000, 100_000], 0, 2.0)?;
    Ok(())
}

#[test]
fn test_max_vs_min_exponential() -> Fallible<()> {
    check_rnm_outcome(MaxDivergence, 0., false, vec![1, 2, 3], 2, f64::INFINITY)?;
    check_rnm_outcome(MaxDivergence, 0., true, vec![1, 2, 3], 0, f64::INFINITY)?;
    check_rnm_outcome(MaxDivergence, 1., false, vec![1, 1, 100_000], 2, 2.0)?;
    check_rnm_outcome(MaxDivergence, 1., true, vec![1, 100_000, 100_000], 0, 2.0)?;
    Ok(())
}
