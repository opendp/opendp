use crate::{error::Fallible, measures::ZeroConcentratedDivergence};

use super::*;

#[test]
fn test_noisy_top_k_gumbel() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::new(true);
    let de = make_noisy_top_k(
        input_domain,
        input_metric,
        ZeroConcentratedDivergence,
        1,
        1.,
        false,
    )?;
    let release = de.invoke(&vec![1., 2., 30., 2., 1.])?;
    assert_eq!(release, vec![2]);
    // (1/1)^2 / 8
    assert_eq!(de.map(&1.0)?, 0.125);

    Ok(())
}

#[test]
fn test_noisy_top_k_exponential() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::new_non_nan());
    let input_metric = LInfDistance::default();
    let de = make_noisy_top_k(input_domain, input_metric, MaxDivergence, 1, 1., false)?;
    let release = de.invoke(&vec![1., 2., 30., 2., 1.])?;
    assert_eq!(release, vec![2]);
    assert_eq!(de.map(&1.0)?, 2.0);

    Ok(())
}

fn check_top_k_outcome<M: TopKMeasure>(
    measure: M,
    scale: f64,
    negate: bool,
    input: Vec<i32>,
    expected: Vec<usize>,
) -> Fallible<()> {
    let m_rnm = make_noisy_top_k(
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
fn test_max_vs_min_gumbel_top_k() -> Fallible<()> {
    check_top_k_outcome(
        ZeroConcentratedDivergence,
        0.,
        false,
        vec![1, 2, 3],
        vec![2],
    )?;
    check_top_k_outcome(ZeroConcentratedDivergence, 0., true, vec![1, 2, 3], vec![0])?;
    check_top_k_outcome(
        ZeroConcentratedDivergence,
        1.,
        false,
        vec![1, 1, 100_000],
        vec![2],
    )?;
    check_top_k_outcome(
        ZeroConcentratedDivergence,
        1.,
        true,
        vec![1, 100_000, 100_000],
        vec![0],
    )?;
    Ok(())
}

#[test]
fn test_max_vs_min_exponential_top_k() -> Fallible<()> {
    check_top_k_outcome(MaxDivergence, 0., false, vec![1, 2, 3], vec![2])?;
    check_top_k_outcome(MaxDivergence, 0., true, vec![1, 2, 3], vec![0])?;
    check_top_k_outcome(MaxDivergence, 1., false, vec![1, 1, 100_000], vec![2])?;
    check_top_k_outcome(MaxDivergence, 1., true, vec![1, 100_000, 100_000], vec![0])?;
    Ok(())
}
