use dashu::rational::RBig;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    measures::{MaxDivergence, RenyiDivergence},
    metrics::AbsoluteDistance,
    traits::{SaturatingCast, samplers::sample_discrete_laplace},
};

use super::*;

fn make_test_scorer()
-> Fallible<Measurement<AtomDomain<u32>, AbsoluteDistance<u32>, MaxDivergence, (f64, &'static str)>>
{
    Measurement::new(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<u32>::default(),
        MaxDivergence,
        Function::new_fallible(|arg| {
            let noise = u32::saturating_cast(sample_discrete_laplace(RBig::ONE)?);
            Ok(((*arg + noise) as f64, "arbitrarily typed candidate info"))
        }),
        PrivacyMap::new(|d_in| *d_in as f64),
    )
}

fn make_nan_test_scorer()
-> Fallible<Measurement<AtomDomain<u32>, AbsoluteDistance<u32>, MaxDivergence, (f64, &'static str)>>
{
    Measurement::new(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<u32>::default(),
        MaxDivergence,
        Function::new_fallible(|_| Ok((f64::NAN, "arbitrarily typed candidate info"))),
        PrivacyMap::new(|d_in| *d_in as f64),
    )
}

fn make_rdp_test_scorer() -> Fallible<
    Measurement<AtomDomain<u32>, AbsoluteDistance<u32>, RenyiDivergence, (f64, &'static str)>,
> {
    Measurement::new(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<u32>::default(),
        RenyiDivergence,
        Function::new_fallible(|arg| Ok((*arg as f64, "arbitrarily typed candidate info"))),
        PrivacyMap::new(|d_in| {
            let d_in = *d_in as f64;
            Function::new(move |alpha: &f64| d_in * alpha / 2.0)
        }),
    )
}

fn make_small_rdp_test_scorer() -> Fallible<
    Measurement<AtomDomain<u32>, AbsoluteDistance<u32>, RenyiDivergence, (f64, &'static str)>,
> {
    Measurement::new(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<u32>::default(),
        RenyiDivergence,
        Function::new_fallible(|arg| Ok((*arg as f64, "arbitrarily typed candidate info"))),
        PrivacyMap::new(|d_in| {
            let d_in = *d_in as f64;
            Function::new(move |alpha: &f64| d_in * alpha / 100.0)
        }),
    )
}

#[test]
fn test_make_select_private_candidate_threshold_max_divergence() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let threshold = 12.0;
    let m_select =
        make_select_private_candidate(m_score, 100.0, Some(threshold), Repetitions::Geometric)?;

    (0..10).try_for_each(|_| match m_select.invoke(&10)? {
        Some((score, _)) if score < threshold => fallible!(
            FailedFunction,
            "returned score must never be below threshold"
        ),
        _ => Ok(()),
    })?;

    assert_eq!(m_select.map(&1)?, 2.0);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_with_nan() -> Fallible<()> {
    let m_score = make_nan_test_scorer()?;
    let threshold = 12.0;
    let m_select =
        make_select_private_candidate(m_score, 100.0, Some(threshold), Repetitions::Geometric)?;
    (0..10).try_for_each(|_| match m_select.invoke(&10)? {
        Some((score, _)) if score < threshold => fallible!(
            FailedFunction,
            "returned score must never be below threshold"
        ),
        _ => Ok(()),
    })?;

    assert_eq!(m_select.map(&1)?, 2.0);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_best_of_geometric() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let m_select = make_select_private_candidate(m_score, 2.0, None, Repetitions::Geometric)?;

    assert!(m_select.invoke(&10)?.is_some());
    assert_eq!(m_select.map(&1)?, 3.0);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_best_of_logarithmic_max_divergence() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let m_select = make_select_private_candidate(m_score, 2.0, None, Repetitions::Logarithmic)?;

    assert!(m_select.invoke(&10)?.is_some());
    assert_eq!(m_select.map(&1)?, 2.0);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_renyi_threshold() -> Fallible<()> {
    let m_score = make_rdp_test_scorer()?;
    let threshold = 12.0;
    let gamma = 0.01;
    let m_select = make_select_private_candidate(
        m_score,
        1.0 / gamma,
        Some(threshold),
        Repetitions::Geometric,
    )?;

    assert_eq!(m_select.map(&1)?.eval(&2.0)?, f64::INFINITY);
    let expected = 2.0 + (2.0 / 3.0) * 1.5 + 2.0 * (1.0 / gamma).ln() / 3.0;
    assert!((m_select.map(&1)?.eval(&4.0)? - expected).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_renyi_best_of_negative_binomial() -> Fallible<()> {
    let m_score = make_rdp_test_scorer()?;
    let gamma = 0.5;
    let mean = 1.0 / gamma;
    let m_select = make_select_private_candidate(m_score, mean, None, Repetitions::Geometric)?;

    assert!(m_select.invoke(&10)?.is_some());
    let expected =
        2.0 + 2.0 * (1.0 - 1.0 / 4.0) * 2.0 + 2.0 * (1.0 / gamma).ln() / 4.0 + mean.ln() / 3.0;
    assert!((m_select.map(&1)?.eval(&4.0)? - expected).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_renyi_poisson() -> Fallible<()> {
    let m_score = make_small_rdp_test_scorer()?;
    let mean = 0.5;
    let m_select = make_select_private_candidate(m_score, mean, None, Repetitions::Poisson)?;

    let curve = m_select.map(&1)?;
    let eps_alpha = 4.0 / 100.0;
    let delta = (1.0 / 4.0) * (3.0 / 4.0f64).powf(3.0);
    let expected = eps_alpha + mean * delta + mean.ln() / 3.0;
    assert!((curve.eval(&4.0)? - expected).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_make_select_private_candidate_rejects_unsupported_combinations() -> Fallible<()> {
    let m_max = make_test_scorer()?;
    assert!(
        make_select_private_candidate(m_max.clone(), 2.0, Some(10.0), Repetitions::Logarithmic,)
            .is_err()
    );
    assert!(
        make_select_private_candidate(m_max.clone(), 2.0, None, Repetitions::Poisson,).is_err()
    );
    assert!(make_select_private_candidate(
        m_max,
        1.0,
        None,
        Repetitions::NegativeBinomial { eta: 1.0 },
    )
    .is_err());

    let m_rdp = make_rdp_test_scorer()?;
    assert!(
        make_select_private_candidate(m_rdp.clone(), 2.0, Some(10.0), Repetitions::Poisson,)
            .is_err()
    );
    assert!(
        make_select_private_candidate(m_rdp, 2.0, Some(10.0), Repetitions::Logarithmic,).is_err()
    );
    Ok(())
}
