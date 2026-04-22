use super::sample_logarithmic_exp;
use crate::{
    error::Fallible,
    traits::samplers::test::{BASE_N, check_chi_square},
};

use dashu::{integer::UBig, rbig};
use std::convert::TryFrom;

fn histogram_ubig(
    mut sampler: impl FnMut() -> Fallible<UBig>,
    n: usize,
    start: u64,
    explicit_bins: usize,
) -> Vec<u64> {
    let mut observed = vec![0u64; explicit_bins + 1];

    for _ in 0..n {
        let sample = u64::try_from(sampler().unwrap()).unwrap();
        assert!(
            sample >= start,
            "sample {sample} fell below lower support bound {start}"
        );

        let rel = sample - start;
        let idx = usize::try_from(rel).unwrap().min(explicit_bins);
        observed[idx] += 1;
    }

    observed
}

fn logarithmic_expected_counts(n: usize, x: f64, explicit_bins: usize) -> Vec<f64> {
    let lambda = (-x).exp();
    let log_norm = (1.0 / (1.0 - lambda)).ln();

    let mut expected = Vec::with_capacity(explicit_bins + 1);

    // p_1 = lambda / log(1 / (1 - lambda))
    let mut pk = lambda / log_norm;
    let mut mass = 0.0;

    // explicit bins for k = 1, 2, ..., explicit_bins
    for k in 1..=explicit_bins {
        expected.push(n as f64 * pk);
        mass += pk;

        let kf = k as f64;
        pk *= lambda * kf / (kf + 1.0);
    }

    // tail bin for k >= explicit_bins + 1
    expected.push((n as f64) * (1.0 - mass));
    expected
}

fn empirical_mean(samples: &[u64]) -> f64 {
    samples.iter().map(|&x| x as f64).sum::<f64>() / samples.len() as f64
}

fn logarithmic_mean(x: f64) -> f64 {
    let lambda = (-x).exp();
    lambda / ((1.0 - lambda) * (1.0 / (1.0 - lambda)).ln())
}

#[test]
fn test_logarithmic_exp_positive_support() {
    for _ in 0..1024 {
        let sample = sample_logarithmic_exp(rbig!(1)).unwrap();
        assert!(!sample.is_zero());
    }
}

#[test]
fn test_logarithmic_exp_chi_square_x_1() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 8;

    let observed = histogram_ubig(|| sample_logarithmic_exp(rbig!(1)), n, 1, explicit_bins);

    let expected = logarithmic_expected_counts(n, 1.0, explicit_bins);
    check_chi_square(&observed, &expected)
}

#[test]
fn test_logarithmic_exp_chi_square_x_half() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 8;

    let observed = histogram_ubig(|| sample_logarithmic_exp(rbig!(1 / 2)), n, 1, explicit_bins);

    let expected = logarithmic_expected_counts(n, 0.5, explicit_bins);
    check_chi_square(&observed, &expected)
}

#[test]
fn test_logarithmic_exp_mean_x_1() {
    let n = BASE_N;
    let samples: Vec<u64> = (0..n)
        .map(|_| {
            let sample = sample_logarithmic_exp(rbig!(1)).unwrap();
            u64::try_from(sample).unwrap()
        })
        .collect();

    let observed = empirical_mean(&samples);
    let expected = logarithmic_mean(1.0);

    // variance is finite, but keep this test simple and stable
    let tol = 0.05;
    assert!(
        (observed - expected).abs() <= tol,
        "mean mismatch: observed={observed}, expected={expected}, tol={tol}"
    );
}
