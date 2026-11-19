use crate::traits::samplers::test::{ALPHA, BASE_N, run_wilson_test};

use super::*;
use dashu::rbig;

#[test]
fn bernoulli_u64_threshold_boundaries() -> Fallible<()> {
    assert_eq!(bernoulli_u64_threshold(0.5), (1u64 << 63) - 1);
    assert_eq!(bernoulli_u64_threshold(0.75), (3u64 << 62) - 1);
    assert_eq!(bernoulli_u64_threshold(1.0), u64::MAX);
    assert_eq!(bernoulli_u64_threshold(f64::MIN_POSITIVE), 0);
    assert_eq!(bernoulli_u64_threshold(0.0), 0);
    assert_eq!(bernoulli_u64_threshold(2f64.powi(-64)), 0);
    assert_eq!(bernoulli_u64_threshold(2f64.powi(-63)), 1);
    assert_eq!(
        bernoulli_u64_threshold(f64::from_bits(0.5f64.to_bits() + 1)),
        (1u64 << 63) + 2047
    );
    assert!(sample_bernoulli_u64(u64::MAX)?);
    Ok(())
}

pub const N_RATIONAL: usize = 2 * BASE_N;

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
        rbig!(1 / 2),
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
