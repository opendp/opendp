use super::{sample_poisson, sample_poisson_0_1};
use crate::{
    error::Fallible,
    traits::samplers::test::{BASE_N, assert_close_normal, check_chi_square},
};

use dashu::{integer::UBig, rbig};
use std::convert::TryFrom;

fn histogram_ubig(
    mut sampler: impl FnMut() -> Fallible<UBig>,
    n: usize,
    explicit_bins: usize,
) -> Vec<u64> {
    let mut observed = vec![0u64; explicit_bins + 1];

    for _ in 0..n {
        let sample = u64::try_from(sampler().unwrap()).unwrap();
        let idx = usize::try_from(sample).unwrap().min(explicit_bins);
        observed[idx] += 1;
    }

    observed
}

fn poisson_expected_counts(n: usize, lambda: f64, explicit_bins: usize) -> Vec<f64> {
    let mut expected = Vec::with_capacity(explicit_bins + 1);

    let mut pk = (-lambda).exp();
    let mut mass = 0.0;

    // explicit bins for k = 0, 1, ..., explicit_bins - 1
    for k in 0..explicit_bins {
        expected.push(n as f64 * pk);
        mass += pk;

        let kf = k as f64;
        pk *= lambda / (kf + 1.0);
    }

    // tail bin for k >= explicit_bins
    expected.push((n as f64) * (1.0 - mass));
    expected
}

fn empirical_mean(samples: &[u64]) -> f64 {
    samples.iter().map(|&x| x as f64).sum::<f64>() / samples.len() as f64
}

#[test]
fn test_poisson_zero() -> Fallible<()> {
    for _ in 0..1024 {
        assert_eq!(sample_poisson(rbig!(0))?, UBig::ZERO);
        assert_eq!(sample_poisson_0_1(rbig!(0))?, UBig::ZERO);
    }
    Ok(())
}

#[test]
fn test_poisson_0_1_chi_square_half() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 7;

    let observed = histogram_ubig(|| sample_poisson_0_1(rbig!(1 / 2)), n, explicit_bins);
    let expected = poisson_expected_counts(n, 0.5, explicit_bins);

    check_chi_square(&observed, &expected)
}

#[test]
fn test_poisson_chi_square_three_halves() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 9;

    let observed = histogram_ubig(|| sample_poisson(rbig!(3 / 2)), n, explicit_bins);
    let expected = poisson_expected_counts(n, 1.5, explicit_bins);

    check_chi_square(&observed, &expected)
}

#[test]
fn test_poisson_mean_three_halves() {
    let n = BASE_N;
    let samples: Vec<u64> = (0..n)
        .map(|_| u64::try_from(sample_poisson(rbig!(3 / 2)).unwrap()).unwrap())
        .collect();

    let est = empirical_mean(&samples);
    let target = 1.5;
    let se = (target / n as f64).sqrt();

    assert_close_normal(est, target, se, "Poisson(3/2) mean");
}
