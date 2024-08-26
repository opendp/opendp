use dashu::rational::RBig;

use crate::{
    domains::AtomDomain,
    metrics::AbsoluteDistance,
    traits::{samplers::sample_discrete_laplace, SaturatingCast},
};

use super::*;

fn make_test_scorer(
) -> Fallible<Measurement<AtomDomain<u32>, (f64, &'static str), AbsoluteDistance<u32>, MaxDivergence>>
{
    Measurement::new(
        AtomDomain::<u32>::default(),
        Function::new_fallible(|arg| {
            let noise = u32::saturating_cast(sample_discrete_laplace(RBig::ONE)?);
            Ok(((*arg + noise) as f64, "arbitrarily typed candidate info"))
        }),
        AbsoluteDistance::<u32>::default(),
        MaxDivergence,
        PrivacyMap::new(|d_in| *d_in as f64),
    )
}

#[test]
fn test_make_select_private_candidate_without_max_iters() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let threshold = 12.0;
    let stop_probability = 0.01;
    let m_select = make_select_private_candidate(m_score, stop_probability, threshold, None)?;
    (0..10).try_for_each(|_| match m_select.invoke(&10)? {
        Some((score, _)) if score < threshold => fallible!(
            FailedFunction,
            "returned score must never be below threshold"
        ),
        _ => Ok(()),
    })?;

    // This constant comes from:
    // 2 * m_score.map(d_in) + 0
    // 2 * 1
    assert_eq!(m_select.map(&1)?, 2.0);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_with_max_iters() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let threshold = 12.0;
    let max_iterations = 40;
    let stop_probability = 0.01;
    let m_select =
        make_select_private_candidate(m_score, stop_probability, threshold, Some(max_iterations))?;
    (0..10).try_for_each(|_| match m_select.invoke(&10)? {
        Some((score, _)) if score < threshold => fallible!(
            FailedFunction,
            "returned score must never be below threshold"
        ),
        _ => Ok(()),
    })?;

    // This constant comes from:
    // 2 * m_score.map(d_in) + 2 / exp(max_iterations * stop_probability)
    // 2 * 1 + 2 / exp(40 * .01)
    assert_eq!(m_select.map(&1)?, 3.340640092071279);
    Ok(())
}
