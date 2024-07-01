use super::*;
use crate::traits::samplers::test::*;

#[test]
fn test_bernoulli() {
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
