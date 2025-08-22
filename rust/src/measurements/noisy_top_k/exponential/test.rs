use std::array::from_fn;

use dashu::rbig;

use crate::traits::samplers::{Shuffle, test::check_chi_square};

use super::*;

fn argsort<T: Ord>(x: &[T]) -> Vec<usize> {
    let mut indices = (0..x.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &x[i]);
    indices
}

#[test]
fn test_peel_permute_and_flip() {
    for len in [0, 1, 2, 3, 4, 5] {
        for _trial in 0..len.min(1) {
            for scale in [rbig![0], rbig![1]] {
                let mut x = vec![rbig![0], rbig![50], rbig![100], rbig![150]];
                x.truncate(len.min(x.len()));
                x.shuffle().unwrap();

                let mut expected = argsort(&x);
                expected.reverse();

                let observed = peel_permute_and_flip(x, scale, len).unwrap();
                assert_eq!(expected, observed);
            }
        }
    }
}

#[test]
fn test_permute_and_flip() {
    for scale in [rbig![0], rbig![1]] {
        for _ in 0..100 {
            let x = [rbig![100], rbig![0], rbig![0]];
            let selection = permute_and_flip(&x, &scale).unwrap();
            assert_eq!(selection, 0);
        }
        assert_eq!(permute_and_flip(&[rbig![0]], &scale).unwrap(), 0);
        assert!(permute_and_flip(&[], &scale).is_err());
    }
}

#[test]
fn test_permute_and_flip_distribution_zero() -> Fallible<()> {
    let scores = vec![rbig!(0).clone(); 10];
    let mut observed = [0.0; 10];
    (0..1000).try_for_each(|_| {
        observed[permute_and_flip(&scores, &rbig!(0))?] += 1.0;
        Fallible::Ok(())
    })?;
    let mut expected = [0.0; 10];
    expected[0] = 1000.0;
    check_chi_square(observed, expected)
}

#[test]
fn test_permute_and_flip_distribution_uniform() -> Fallible<()> {
    let scores = vec![rbig!(0).clone(); 10];
    let mut observed = [0.0; 10];
    (0..1000).try_for_each(|_| {
        observed[permute_and_flip(&scores, &rbig!(1))?] += 1.0;
        Fallible::Ok(())
    })?;
    check_chi_square(observed, [100.0; 10])
}

#[test]
fn test_permute_and_flip_distribution_varied() -> Fallible<()> {
    let scores: [_; 10] = from_fn(RBig::from);
    let trials = 10000;
    let mut observed = [0.0; 10];
    (0..trials).try_for_each(|_| {
        observed[permute_and_flip(&scores, &rbig!(1))?] += 1.0 / trials as f64;
        Fallible::Ok(())
    })?;

    let expected = permute_and_flip_pmf(
        &scores
            .into_iter()
            .map(|r| r.to_f64().value())
            .collect::<Vec<_>>(),
        1.0,
    )
    .try_into()
    .unwrap();

    check_chi_square(observed, expected)
}

/// Implements the PMF of the permute and flip algorithm,
/// from https://arxiv.org/pdf/2010.12603#appendix.E
fn permute_and_flip_pmf(scores: &[f64], scale: f64) -> Vec<f64> {
    let n = scores.len();

    // Calculate p: exp((scores - max(scores)) / scale)
    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let p: Vec<f64> = scores
        .iter()
        .map(|&s| ((s - max_score) / scale).exp()) // Apply element-wise exponential
        .collect();

    fn s(k: usize, r: usize, p: &[f64]) -> f64 {
        if k == 0 {
            return 1.0;
        }
        if r == 0 {
            return 0.0;
        }
        s(k, r - 1, p) + p[r - 1] * s(k - 1, r - 1, p)
    }

    fn t(k: usize, r: usize, n: usize, p: &[f64]) -> f64 {
        if k == 0 {
            return 1.0;
        }
        s(k, n, p) - p[r - 1] * t(k - 1, r, n, p)
    }

    fn sign(i: usize) -> f64 {
        if i % 2 == 0 { 1.0 } else { -1.0 }
    }

    // Calculate mass for a given r
    let mass = |r: usize| {
        let sum = (0..n)
            .map(|k| sign(k) / ((k + 1) as f64) * t(k, r, n, &p))
            .sum::<f64>();
        p[r - 1] * sum
    };

    (1..=n).map(mass).collect()
}
