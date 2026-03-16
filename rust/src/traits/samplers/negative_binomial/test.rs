use super::{
    sample_negative_binomial_integer, sample_negative_binomial_rational,
    sample_truncated_negative_binomial_rational,
};
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

fn ordinary_nb_expected_counts(n: usize, eta: f64, x: f64, explicit_bins: usize) -> Vec<f64> {
    let q = (-x).exp();
    let p = 1.0 - q;

    // P[K = 0]
    let mut pk = p.powf(eta);
    let mut expected = Vec::with_capacity(explicit_bins + 1);
    let mut mass = 0.0;

    // explicit bins for k = 0, 1, ..., explicit_bins - 1
    for k in 0..explicit_bins {
        expected.push(n as f64 * pk);
        mass += pk;

        let kf = k as f64;
        pk *= q * (eta + kf) / (kf + 1.0);
    }

    // tail bin for k >= explicit_bins
    expected.push((n as f64) * (1.0 - mass));
    expected
}

fn truncated_nb_expected_counts(n: usize, eta: f64, x: f64, explicit_bins: usize) -> Vec<f64> {
    let q = (-x).exp();
    let p = 1.0 - q;
    let p0 = p.powf(eta);
    let mass_pos = 1.0 - p0;

    // first positive mass: P[K = 1] for ordinary NB
    let mut pk = p0 * q * eta;
    let mut expected = Vec::with_capacity(explicit_bins + 1);
    let mut mass = 0.0;

    // explicit bins for k = 1, 2, ..., explicit_bins
    for k in 1..=explicit_bins {
        let ptk = pk / mass_pos;
        expected.push(n as f64 * ptk);
        mass += ptk;

        let kf = k as f64;
        pk *= q * (eta + kf) / (kf + 1.0);
    }

    // tail bin for k >= explicit_bins + 1
    expected.push((n as f64) * (1.0 - mass));
    expected
}

#[test]
fn test_negative_binomial_integer_exp_chi_square() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 8;

    let observed = histogram_ubig(
        || sample_negative_binomial_integer(UBig::from(3u8), rbig!(1)),
        n,
        0,
        explicit_bins,
    );

    let expected = ordinary_nb_expected_counts(n, 3.0, 1.0, explicit_bins);
    check_chi_square(&observed, &expected)
}

#[test]
fn test_negative_binomial_exp_envelope_integer_eta_chi_square() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 8;

    let observed = histogram_ubig(
        || sample_negative_binomial_rational(rbig!(3), rbig!(1)),
        n,
        0,
        explicit_bins,
    );

    let expected = ordinary_nb_expected_counts(n, 3.0, 1.0, explicit_bins);
    check_chi_square(&observed, &expected)
}

#[test]
fn test_negative_binomial_exp_envelope_fractional_eta_chi_square() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 8;

    let observed = histogram_ubig(
        || sample_negative_binomial_rational(rbig!(3 / 2), rbig!(1)),
        n,
        0,
        explicit_bins,
    );

    let expected = ordinary_nb_expected_counts(n, 1.5, 1.0, explicit_bins);
    check_chi_square(&observed, &expected)
}

#[test]
fn test_truncated_negative_binomial_exp_envelope_fractional_eta_chi_square() -> Fallible<()> {
    let n = BASE_N;
    let explicit_bins = 8;

    let observed = histogram_ubig(
        || sample_truncated_negative_binomial_rational(rbig!(3 / 2), rbig!(1)),
        n,
        1,
        explicit_bins,
    );

    let expected = truncated_nb_expected_counts(n, 1.5, 1.0, explicit_bins);
    check_chi_square(&observed, &expected)
}

#[test]
fn test_truncated_negative_binomial_exp_envelope_has_positive_support() {
    for _ in 0..1024 {
        let sample = sample_truncated_negative_binomial_rational(rbig!(3 / 2), rbig!(1)).unwrap();
        assert!(!sample.is_zero());
    }
}
