use dashu::rbig;

use crate::traits::samplers::cks20::sample_bernoulli_exp1;
use crate::traits::samplers::sample_bernoulli_exp;
use crate::traits::samplers::test::{ALPHA, run_wilson_test};
use crate::traits::samplers::test::{FP_SLOP, TEST_Z, assert_close_normal, sample_mean_bool};

use super::*;

pub const N_ENDPOINTS: usize = 1_000;

fn test_fixed_points_wilson(
    sampler: impl Fn(RBig) -> bool,
    xs: &[RBig],
    n: usize,
    alpha: f64,
    label: &str,
) {
    for x in xs {
        let p0 = p_exp_neg(x);
        run_wilson_test(|| sampler(x.clone()), p0, n, alpha, label);
    }
}

/// Collect empirical p_hat and theoretical p for monotonicity tests
fn collect_phat_and_p<S>(sampler: S, xs: &[RBig], n: usize) -> Vec<(f64, f64)>
where
    S: Fn(RBig) -> bool,
{
    xs.iter()
        .map(|x| {
            let p = p_exp_neg(x);
            let p_hat = sample_mean_bool(|| sampler(x.clone()), n);
            (p_hat, p)
        })
        .collect()
}

pub fn assert_monotone_decreasing(phats_and_ps: &[(f64, f64)], n: usize) {
    for i in 0..(phats_and_ps.len() - 1) {
        let (p_hat_a, p_a) = phats_and_ps[i];
        let (p_hat_b, p_b) = phats_and_ps[i + 1];

        let se_a = (p_a * (1.0 - p_a) / (n as f64)).sqrt();
        let se_b = (p_b * (1.0 - p_b) / (n as f64)).sqrt();
        let band = TEST_Z * (se_a * se_a + se_b * se_b).sqrt() + FP_SLOP;

        assert!(
            p_hat_a + band >= p_hat_b,
            "monotonicity violated: p_hat[{}]={} < p_hat[{}]={} beyond band {}",
            i,
            p_hat_a,
            i + 1,
            p_hat_b,
            band
        );
    }
}

/// Check that exp(−(a+b))=exp(−a)exp(−b) for the bernoulli sampler S.
fn assert_factorizes_addition<S>(sampler: S, a: &RBig, b: &RBig, n: usize)
where
    S: Fn(RBig) -> bool,
{
    let ab = a.clone() + b.clone();

    let p_a_hat = sample_mean_bool(|| sampler(a.clone()), n);
    let p_b_hat = sample_mean_bool(|| sampler(b.clone()), n);
    let p_ab_hat = sample_mean_bool(|| sampler(ab.clone()), n);

    // Use theoretical p only to compute uncertainty bands.
    let p_a = p_exp_neg(a);
    let p_b = p_exp_neg(b);
    let p_ab = p_exp_neg(&ab);

    let se_ab = (p_ab * (1.0 - p_ab) / (n as f64)).sqrt();
    let se_a = (p_a * (1.0 - p_a) / (n as f64)).sqrt();
    let se_b = (p_b * (1.0 - p_b) / (n as f64)).sqrt();

    let prod_hat = p_a_hat * p_b_hat;
    let se_prod = ((p_b * p_b) * (se_a * se_a) + (p_a * p_a) * (se_b * se_b)).sqrt();

    let se_diff = (se_ab * se_ab + se_prod * se_prod).sqrt();
    assert_close_normal(p_ab_hat, prod_hat, se_diff, "factorization");
}

// ------------------------------------------------------------
// exp1 tests
// ------------------------------------------------------------

#[test]
fn exp1_matches_exp_neg_x_fixed_points_wilson() {
    let xs = vec![
        rbig!(0 / 1),
        rbig!(1 / 16),
        rbig!(1 / 8),
        rbig!(1 / 4),
        rbig!(1 / 2),
        rbig!(3 / 4),
        rbig!(1 / 1),
    ];

    let sampler = |x: RBig| sample_bernoulli_exp1(x).unwrap();
    test_fixed_points_wilson(sampler, &xs, N_BERNOULLI, ALPHA, "exp1 fixed point");
}

#[test]
fn exp1_is_monotone_decreasing() {
    let xs = vec![
        rbig!(0 / 1),
        rbig!(1 / 8),
        rbig!(1 / 4),
        rbig!(3 / 8),
        rbig!(1 / 2),
        rbig!(3 / 4),
        rbig!(1 / 1),
    ];

    let sampler = |x: RBig| sample_bernoulli_exp1(x).unwrap();
    let stats = collect_phat_and_p(sampler, &xs, N_BERNOULLI);
    assert_monotone_decreasing(&stats, N_BERNOULLI);
}

#[test]
fn exp1_endpoints() {
    let sampler = |x: RBig| sample_bernoulli_exp1(x).unwrap();

    let x0 = rbig!(0 / 1);
    for _ in 0..N_ENDPOINTS {
        assert!(sampler(x0.clone()));
    }

    // For x=1 use Wilson instead of mean-close.
    let x1 = rbig!(1 / 1);
    let p0 = p_exp_neg(&x1);
    run_wilson_test(
        || sampler(x1.clone()),
        p0,
        N_BERNOULLI,
        ALPHA,
        "exp1 endpoint x=1",
    );
}

// ------------------------------------------------------------
// exp tests
// ------------------------------------------------------------

#[test]
fn exp_matches_exp_neg_x_fixed_points_wilson() {
    let xs = vec![
        rbig!(0 / 1),
        rbig!(1 / 8),
        rbig!(3 / 4),
        rbig!(1 / 1),
        rbig!(3 / 2),
        rbig!(2),
        rbig!(9 / 4),
        rbig!(3),
        rbig!(7 / 2),
        rbig!(5),
    ];

    let sampler = |x: RBig| sample_bernoulli_exp(x).unwrap();
    test_fixed_points_wilson(sampler, &xs, N_BERNOULLI, ALPHA, "exp fixed point");
}

#[test]
fn exp_factorizes_over_addition() {
    let sampler = |x: RBig| sample_bernoulli_exp(x).unwrap();
    let pairs: Vec<(RBig, RBig)> = vec![
        (rbig!(1 / 2), rbig!(1 / 2)),
        (rbig!(1), rbig!(1 / 4)),
        (rbig!(2), rbig!(3 / 4)),
        (rbig!(5 / 2), rbig!(1 / 8)),
    ];
    for (a, b) in pairs {
        assert_factorizes_addition(&sampler, &a, &b, N_BERNOULLI);
    }
}

#[test]
fn exp_is_monotone_decreasing() {
    let xs = vec![
        rbig!(0 / 1),
        rbig!(1 / 2),
        rbig!(3 / 2),
        rbig!(2),
        rbig!(9 / 4),
        rbig!(3),
        rbig!(4),
        rbig!(6),
    ];

    let sampler = |x: RBig| sample_bernoulli_exp(x).unwrap();
    let stats = collect_phat_and_p(sampler, &xs, N_BERNOULLI);
    assert_monotone_decreasing(&stats, N_BERNOULLI);
}

#[test]
fn exp_endpoints() {
    let sampler = |x: RBig| sample_bernoulli_exp(x).unwrap();

    let x0 = rbig!(0 / 1);
    for _ in 0..N_ENDPOINTS {
        assert!(sampler(x0.clone()));
    }

    let x1 = rbig!(1);
    let p0 = p_exp_neg(&x1);
    run_wilson_test(
        || sampler(x1.clone()),
        p0,
        N_BERNOULLI,
        ALPHA,
        "exp endpoint x=1",
    );
}

fn assert_factorizes_addition_empirical(a: &RBig, b: &RBig, n: usize) {
    let ab = a.clone() + b.clone();

    let p_ab_hat = sample_mean_bool(|| sample_bernoulli_exp(ab.clone()).unwrap(), n);

    let p_and_hat = sample_mean_bool(
        || sample_bernoulli_exp(a.clone()).unwrap() && sample_bernoulli_exp(b.clone()).unwrap(),
        n,
    );

    let se_diff = 1.0 / (2.0_f64 * (n as f64)).sqrt();
    assert_close_normal(p_ab_hat, p_and_hat, se_diff, "factorization (large denom)");
}

fn assert_scaling_identity_empirical(x: &RBig, m: u32, n: usize) {
    assert!(m >= 2);
    let mx = x.clone() * RBig::from(m);

    let p_mx_hat = sample_mean_bool(|| sample_bernoulli_exp(mx.clone()).unwrap(), n);

    let p_and_hat = sample_mean_bool(
        || {
            let mut ok = true;
            for _ in 0..m {
                ok &= sample_bernoulli_exp(x.clone()).unwrap();
                if !ok {
                    break;
                }
            }
            ok
        },
        n,
    );

    let se_diff = 1.0 / (2.0_f64 * (n as f64)).sqrt();
    assert_close_normal(
        p_mx_hat,
        p_and_hat,
        se_diff,
        "scaling identity (large denom)",
    );
}

fn big_denom_rationals_in_0_1() -> Vec<RBig> {
    vec![
        RBig::from(1i64) / RBig::from(1i64 << 30),
        RBig::from(3i64) / RBig::from(1i64 << 30),
        RBig::from(1i64) / RBig::from(65_537i64),
        RBig::from(32_768i64) / RBig::from(65_537i64),
        RBig::from(65_536i64) / RBig::from(65_537i64),
        RBig::from(12_345i64) / RBig::from(1_000_003i64),
        RBig::from(999_983i64) / RBig::from(1_000_003i64),
    ]
}

#[test]
fn exp_factorization_holds_for_large_denominators() {
    let n = N_BERNOULLI;
    let xs = big_denom_rationals_in_0_1();

    let pairs: Vec<(RBig, RBig)> = vec![
        (xs[0].clone(), xs[2].clone()),
        (xs[1].clone(), xs[5].clone()),
        (xs[3].clone(), xs[2].clone()),
        (xs[4].clone(), xs[0].clone()),
    ];

    for (a, b) in pairs {
        assert_factorizes_addition_empirical(&a, &b, n);
    }
}

#[test]
fn exp_scaling_identity_holds_for_large_denominators() {
    let n = N_BERNOULLI;

    let x1 = RBig::from(1i64) / RBig::from(65_537i64);
    let x2 = RBig::from(12_345i64) / RBig::from(1_000_003i64);

    assert_scaling_identity_empirical(&x1, 7, n);
    assert_scaling_identity_empirical(&x2, 13, n);
}
