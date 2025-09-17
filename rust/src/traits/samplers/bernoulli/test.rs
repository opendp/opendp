use dashu::rbig;

use super::*;
use crate::traits::samplers::test::*;

#[test]
fn test_bernoulli_float() {
    [0.2, 0.5, 0.7, 0.9].iter().for_each(|p| {
        let sampler = || {
            if sample_bernoulli_float(*p, false).unwrap() {
                1.
            } else {
                0.
            }
        };
        assert!(
            test_proportion_parameters(sampler, *p, 0.00001, *p / 100.),
            "empirical evaluation of the bernoulli({:?}) distribution failed",
            p
        )
    })
}

#[test]
fn test_bernoulli_rational() {
    [rbig!(1 / 5), rbig!(1 / 2), rbig!(7 / 10), rbig!(9 / 10)]
        .iter()
        .for_each(|p| {
            let sampler = || {
                if sample_bernoulli_rational(p.clone()).unwrap() {
                    1.
                } else {
                    0.
                }
            };
            let f_p = p.to_f64().value();
            assert!(
                test_proportion_parameters(sampler, f_p, 0.00001, f_p / 100.),
                "empirical evaluation of the bernoulli({:?}) distribution failed",
                p
            )
        })
}
