use crate::traits::samplers::test::{
    ALPHA, BASE_N, assert_close_normal, run_wilson_test, sample_mean_bool,
};

use super::*;
use dashu::rbig;

pub const N_RATIONAL: usize = 2 * BASE_N;
pub const N_FLOAT: usize = 3 * BASE_N;

#[test]
fn bernoulli_rational_boundaries() -> Fallible<()> {
    assert!(!sample_bernoulli_rational(rbig!(0))?);
    assert!(sample_bernoulli_rational(rbig!(1))?);
    Ok(())
}

#[test]
fn bernoulli_rational_rejects_invalid_inputs() {
    assert!(sample_bernoulli_rational(rbig!(-1 / 2)).is_err());
    assert!(sample_bernoulli_rational(rbig!(3 / 2)).is_err());
}

#[test]
fn wilson_rational_fixed_points() {
    let ps = vec![
        rbig!(0 / 1),
        rbig!(1 / 16),
        rbig!(1 / 8),
        rbig!(1 / 4),
        rbig!(3 / 8),
        rbig!(1 / 2),
        rbig!(3 / 4),
        rbig!(15 / 16),
        rbig!(1 / 1),
    ];

    for p in ps {
        let p0 = p.to_f64().value();
        run_wilson_test(
            || sample_bernoulli_rational(p.clone()).unwrap(),
            p0,
            N_RATIONAL,
            ALPHA,
            "sample_bernoulli_rational",
        );
    }
}

fn run_wilson_test_float<T: Float>(p: T, constant_time: bool, n: usize, label: &str)
where
    T::Bits: ExactIntCast<usize>,
    usize: ExactIntCast<T::Bits>,
{
    run_wilson_test(
        || sample_bernoulli_float::<T>(p, constant_time).unwrap(),
        p.to_f64().unwrap(),
        n,
        ALPHA,
        label,
    );
}

#[test]
fn bernoulli_float_boundaries_f64() -> Fallible<()> {
    for _ in 0..20_000 {
        assert!(!sample_bernoulli_float::<f64>(0.0, false)?);
        assert!(!sample_bernoulli_float::<f64>(0.0, true)?);
        assert!(sample_bernoulli_float::<f64>(1.0, false)?);
        assert!(sample_bernoulli_float::<f64>(1.0, true)?);
    }
    Ok(())
}

#[test]
fn wilson_float_f64_fixed_points() {
    let ps: Vec<f64> = vec![0.0, 0.5, 0.25, 0.125, 0.75, 0.1, 0.2, 0.3, 0.7, 0.99];
    for &p in &ps {
        run_wilson_test_float::<f64>(p, false, N_FLOAT, "bernoulli_float f64 non-ct");
        run_wilson_test_float::<f64>(p, true, N_FLOAT, "bernoulli_float f64 ct");
    }
}

#[test]
fn wilson_float_f32_fixed_points() {
    let ps: Vec<f32> = vec![0.0, 0.5, 0.25, 0.125, 0.75, 0.1, 0.3, 0.7, 0.99];
    for &p in &ps {
        run_wilson_test_float::<f32>(p, false, N_FLOAT / 2, "bernoulli_float f32 non-ct");
        run_wilson_test_float::<f32>(p, true, N_FLOAT / 2, "bernoulli_float f32 ct");
    }
}

#[test]
fn bernoulli_float_constant_time_and_nonconstant_match_f64() {
    // Compare ct vs non-ct at same p via a mean-difference test,
    // using the existing helpers/constants from testutil.
    let ps: [f64; 6] = [0.01, 0.1, 0.3, 0.5, 0.9, 0.99];

    for &p in &ps {
        let m_ct = sample_mean_bool(|| sample_bernoulli_float::<f64>(p, true).unwrap(), N_FLOAT);
        let m_nc = sample_mean_bool(|| sample_bernoulli_float::<f64>(p, false).unwrap(), N_FLOAT);

        // Under the null, both estimators target the same mean p, so
        // SE(diff) = sqrt(Var(hat1)+Var(hat2)) with Var(hat)=p(1-p)/N.
        let se_diff = (2.0 * p * (1.0 - p) / (N_FLOAT as f64)).sqrt();
        assert_close_normal(m_ct, m_nc, se_diff, "ct vs non-ct (f64)");
    }
}

#[test]
fn bernoulli_float_power_of_two_spot_checks_f64() {
    let ps: Vec<f64> = vec![0.5, 0.25, 0.125, 0.0625, 0.03125];
    for &p in &ps {
        run_wilson_test_float::<f64>(p, false, N_FLOAT, "dyadic f64 non-ct");
        run_wilson_test_float::<f64>(p, true, N_FLOAT, "dyadic f64 ct");
    }
}
