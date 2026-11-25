use dashu::{integer::IBig, rational::RBig};

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
            let noisy =
                i64::saturating_cast(IBig::from(*arg) + sample_discrete_laplace(RBig::ONE)?);
            Ok((noisy as f64, "candidate"))
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
        Function::new_fallible(|arg| {
            let noisy =
                i64::saturating_cast(IBig::from(*arg) + sample_discrete_laplace(RBig::ONE)?);
            Ok((noisy as f64, "candidate"))
        }),
        PrivacyMap::new(|d_in| {
            let epsilon = *d_in as f64;
            Function::new(move |_alpha: &f64| epsilon)
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
        Function::new_fallible(|arg| {
            let noisy = i64::saturating_cast(
                IBig::from(*arg) + sample_discrete_laplace(RBig::from(100u8))?,
            );
            Ok((noisy as f64, "candidate"))
        }),
        PrivacyMap::new(|d_in| {
            let epsilon = *d_in as f64 / 100.0;
            Function::new(move |_alpha: &f64| epsilon)
        }),
    )
}

#[test]
fn test_private_selection_threshold_max_divergence() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let threshold = 12.0;
    let m_select = make_select_at_least(m_score, 100.0, threshold)?;

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
fn test_private_selection_with_nan() -> Fallible<()> {
    let m_score = make_nan_test_scorer()?;
    let threshold = 12.0;
    let m_select = make_select_at_least(m_score, 100.0, threshold)?;
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
fn test_private_selection_best_of_geometric() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let m_select = make_select_best_candidate(m_score, 2.0, Repetitions::Geometric)?;

    assert!(m_select.invoke(&10)?.is_some());
    assert_eq!(m_select.map(&1)?, 3.0);
    Ok(())
}

#[test]
fn test_private_selection_best_of_logarithmic_max_divergence() -> Fallible<()> {
    let m_score = make_test_scorer()?;
    let m_select = make_select_best_candidate(m_score, 2.0, Repetitions::Logarithmic)?;

    assert!(m_select.invoke(&10)?.is_some());
    assert_eq!(m_select.map(&1)?, 2.0);
    Ok(())
}

#[test]
fn test_private_selection_renyi_threshold() -> Fallible<()> {
    let m_score = make_rdp_test_scorer()?;
    let threshold = 12.0;
    let gamma = 0.01;
    let m_select = make_select_at_least(m_score, 1.0 / gamma, threshold)?;

    assert_eq!(m_select.map(&1)?.eval(&2.0)?, f64::INFINITY);
    let expected = 1.0 + (2.0 / 3.0) * 1.0 + 2.0 * (1.0 / gamma).ln() / 3.0;
    assert!((m_select.map(&1)?.eval(&4.0)? - expected).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_private_selection_renyi_best_of_negative_binomial() -> Fallible<()> {
    let m_score = make_rdp_test_scorer()?;
    let gamma = 0.5;
    let mean = 1.0 / gamma;
    let m_select = make_select_best_candidate(m_score, mean, Repetitions::Geometric)?;

    assert!(m_select.invoke(&10)?.is_some());
    let expected =
        1.0 + 2.0 * (1.0 - 1.0 / 4.0) * 1.0 + 2.0 * (1.0 / gamma).ln() / 4.0 + mean.ln() / 3.0;
    assert!((m_select.map(&1)?.eval(&4.0)? - expected).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_private_selection_renyi_poisson() -> Fallible<()> {
    let m_score = make_small_rdp_test_scorer()?;
    let mean = 0.5;
    let m_select = make_select_best_candidate(m_score, mean, Repetitions::Poisson)?;

    let curve = m_select.map(&1)?;
    let eps_alpha: f64 = 1.0 / 100.0;
    let expected = mean * (3.0 * eps_alpha).exp_m1() / 3.0;
    assert!((curve.eval(&4.0)? - expected).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_private_selection_mean_one() -> Fallible<()> {
    let scorer = make_test_scorer()?;
    let thresholded = make_select_at_least(scorer.clone(), 1.0, 12.0)?;
    assert_eq!(thresholded.map(&1)?, scorer.map(&1)?);

    for law in [
        Repetitions::Geometric,
        Repetitions::Logarithmic,
        Repetitions::NegativeBinomial { eta: 0.5 },
    ] {
        let selected = make_select_best_candidate(scorer.clone(), 1.0, law)?;
        assert!(selected.invoke(&10)?.is_some());
        assert_eq!(selected.map(&1)?, scorer.map(&1)?);
    }

    let rdp_scorer = make_rdp_test_scorer()?;
    let rdp_selected = make_select_at_least(rdp_scorer.clone(), 1.0, 12.0)?;
    assert_eq!(
        rdp_selected.map(&1)?.eval(&2.0)?,
        rdp_scorer.map(&1)?.eval(&2.0)?
    );

    let rdp_best = make_select_best_candidate(rdp_scorer.clone(), 1.0, Repetitions::Geometric)?;
    assert_eq!(
        rdp_best.map(&1)?.eval(&2.0)?,
        rdp_scorer.map(&1)?.eval(&2.0)?
    );
    Ok(())
}

#[test]
fn test_private_selection_rejects_unsupported_combinations() -> Fallible<()> {
    let m_max = make_test_scorer()?;
    assert!(make_select_best_candidate(m_max.clone(), 2.0, Repetitions::Poisson).is_err());
    assert!(
        make_select_best_candidate(
            m_max.clone(),
            2.0,
            Repetitions::NegativeBinomial { eta: -0.5 }
        )
        .is_err()
    );
    assert!(
        make_select_best_candidate(
            m_max.clone(),
            1.0,
            Repetitions::NegativeBinomial { eta: -0.5 }
        )
        .is_err()
    );
    assert!(make_select_at_least(m_max, f64::NAN, 10.0).is_err());

    let m_rdp = make_rdp_test_scorer()?;
    assert!(make_select_best_candidate(m_rdp.clone(), -1.0, Repetitions::Poisson).is_err());
    assert!(make_select_at_least(m_rdp, 2.0, f64::NAN).is_err());
    Ok(())
}

#[test]
fn test_private_selection_poisson_small_mean_is_valid_rdp() -> Fallible<()> {
    // For neighboring Bernoulli scorers, compare the exact best-of-Poisson
    // divergence against the accountant. The former is strictly positive.
    let alpha: f64 = 4.0;
    let mean: f64 = 0.5;
    let epsilon = (50.0f64 / 49.0).ln();
    let base = Function::new(move |_order: &f64| epsilon);
    let curve = super::bounds::new_poisson_rdp_curve(base, mean);

    let pmf = |p: f64| {
        let empty = (-mean).exp();
        [empty, (-mean * p).exp() - empty, 1.0 - (-mean * p).exp()]
    };
    let p = pmf(0.5);
    let q = pmf(0.51);
    let moment = (0..3)
        .map(|i| p[i].powf(alpha) * q[i].powf(1.0 - alpha))
        .sum::<f64>();
    let exact = moment.ln() / (alpha - 1.0);
    assert!(exact > 0.0);
    assert!(curve.eval(&alpha)? >= exact);
    Ok(())
}

#[test]
#[allow(deprecated)]
fn test_private_selection_legacy_stopping_probability() -> Fallible<()> {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_measurement = calls.clone();

    let scorer = Measurement::new(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<u32>::default(),
        MaxDivergence,
        Function::new(move |_arg| {
            calls_for_measurement.fetch_add(1, Ordering::Relaxed);
            (0.0, "candidate")
        }),
        PrivacyMap::new(|_d_in| 0.0),
    )?;
    let selector = make_select_private_candidate(scorer, 0.01, 1.0)?;
    for _ in 0..64 {
        assert!(selector.invoke(&0)?.is_none());
    }
    // The old inverted logarithm led to ~1 invocation per run.
    assert!(calls.load(Ordering::Relaxed) > 64 * 32);
    Ok(())
}

#[test]
fn test_select_at_least_includes_minimum() -> Fallible<()> {
    let scorer = Measurement::new(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<u32>::default(),
        MaxDivergence,
        Function::new(|_| (12.0, "candidate")),
        PrivacyMap::new(|_| 0.0),
    )?;
    let selected = make_select_at_least(scorer, 1.0, 12.0)?;
    assert_eq!(selected.invoke(&0)?, Some((12.0, "candidate")));
    Ok(())
}
