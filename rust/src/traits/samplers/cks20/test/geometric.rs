use super::*;
use crate::traits::samplers::{
    cks20::sample_geometric_exp_slow,
    sample_geometric_exp_fast,
    test::{assert_close_binomial_mean, assert_close_normal, check_chi_square_from_probs},
};
use dashu::{integer::UBig, rbig};

// Geometric(p) over {0,1,2,...} with P(K=k)=(1-p)^k p.
// Here p = 1 - exp(-x), q = exp(-x).
fn geom_pq(x: &RBig) -> (f64, f64) {
    let q = p_exp_neg(x);
    let p = 1.0 - q;
    (p, q)
}

fn geom_mean_var(x: &RBig) -> (f64, f64) {
    let (p, q) = geom_pq(x);
    let mean = q / p;
    let var = q / (p * p);
    (mean, var)
}

#[derive(Debug, Clone)]
struct GeomSummary {
    n: usize,
    mean: f64,
    p0: f64,
}

fn sample_geom_summary(mut sampler: impl FnMut() -> UBig, n: usize) -> GeomSummary {
    let mut sum = 0.0_f64;
    let mut zeros: u64 = 0;

    for _ in 0..n {
        let k = sampler();
        if k.is_zero() {
            zeros += 1;
        }
        let k_u64 = u64::try_from(k).unwrap();
        sum += k_u64 as f64;
    }

    GeomSummary {
        n,
        mean: sum / (n as f64),
        p0: (zeros as f64) / (n as f64),
    }
}

fn assert_geom_moments(x: &RBig, summ: &GeomSummary) {
    let (mean, var) = geom_mean_var(x);

    let se_mean = (var / (summ.n as f64)).sqrt();
    assert_close_normal(summ.mean, mean, se_mean, "geom mean");

    let (p, _q) = geom_pq(x); // P(K=0) = p
    assert_close_binomial_mean(summ.p0, p, summ.n, "P(K=0)");
}

fn choose_kmax_for_chi2(x: &RBig, n: usize, min_exp: f64, hard_cap: usize) -> usize {
    let (p, q) = geom_pq(x);
    let n_f = n as f64;

    // Want expected count for bin kmax-1 and tail to be >= min_exp, but simplest:
    // ensure expected count for k=kmax-1 (smallest explicit bin) is >= min_exp.
    // That is: n * q^(kmax-1) * p >= min_exp.
    for kmax in (2..=hard_cap).rev() {
        let exp_cnt = n_f * q.powi((kmax as i32) - 1) * p;
        if exp_cnt >= min_exp {
            return kmax;
        }
    }
    2
}

fn assert_binned_chi_square(x: &RBig, n: usize, kmax: usize, mut sampler: impl FnMut() -> UBig) {
    // observed counts for bins: 0..kmax-1 plus tail bin kmax
    let mut observed = vec![0u64; kmax + 1];

    for _ in 0..n {
        let k = sampler();
        let k_u64 = u64::try_from(k).unwrap();
        if (k_u64 as usize) < kmax {
            observed[k_u64 as usize] += 1;
        } else {
            observed[kmax] += 1;
        }
    }

    let (p, q) = geom_pq(x);

    // expected probabilities for same bins
    // P(K=k)=q^k p, tail P(K>=kmax)=q^kmax
    let mut probs = vec![0f64; kmax + 1];
    for k in 0..kmax {
        probs[k] = q.powi(k as i32) * p;
    }
    probs[kmax] = q.powi(kmax as i32);

    check_chi_square_from_probs(&observed, &probs).unwrap()
}

#[test]
fn geometric_fast_x0_is_zero() {
    let x0 = rbig!(0);
    for _ in 0..10_000 {
        let k = sample_geometric_exp_fast(x0.clone()).unwrap();
        assert!(k.is_zero());
    }
}

#[test]
fn geometric_slow_matches_moments_and_p0_at_fixed_points() {
    let xs = vec![rbig!(1 / 2), rbig!(1), rbig!(2), rbig!(3)];

    for x in xs {
        let summ = sample_geom_summary(
            || sample_geometric_exp_slow(x.clone()).unwrap(),
            N_GEOM_SLOW,
        );
        assert_geom_moments(&x, &summ);
    }
}

#[test]
fn geometric_fast_matches_moments_and_p0_at_fixed_points() {
    let xs = vec![
        rbig!(1 / 2),
        rbig!(3 / 4),
        rbig!(1),
        rbig!(3 / 2),
        rbig!(2),
        rbig!(9 / 4),
        rbig!(3),
        rbig!(5),
    ];

    for x in xs {
        let summ = sample_geom_summary(
            || sample_geometric_exp_fast(x.clone()).unwrap(),
            N_GEOM_FAST,
        );
        assert_geom_moments(&x, &summ);
    }
}

#[test]
fn geometric_fast_goodness_of_fit_binned_chi_square() {
    let x = rbig!(1);

    // Choose kmax based on expected counts so the test stays valid if N changes.
    let kmax = choose_kmax_for_chi2(&x, N_GEOM_FAST, 10.0, 20);
    assert_binned_chi_square(&x, N_GEOM_FAST, kmax, || {
        sample_geometric_exp_fast(x.clone()).unwrap()
    });
}

#[test]
fn geometric_fast_and_slow_agree_on_mean_statistically() {
    let xs = vec![rbig!(1 / 2), rbig!(1), rbig!(2)];

    for x in xs {
        let fast = sample_geom_summary(
            || sample_geometric_exp_fast(x.clone()).unwrap(),
            N_GEOM_FAST,
        );
        let slow = sample_geom_summary(
            || sample_geometric_exp_slow(x.clone()).unwrap(),
            N_GEOM_SLOW,
        );

        let (_mean, var) = geom_mean_var(&x);
        let se_fast = (var / (N_GEOM_FAST as f64)).sqrt();
        let se_slow = (var / (N_GEOM_SLOW as f64)).sqrt();
        let se_diff = (se_fast * se_fast + se_slow * se_slow).sqrt();

        assert_close_normal(fast.mean, slow.mean, se_diff, "fast vs slow mean");
    }
}
