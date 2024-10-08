use crate::{
    error::Fallible,
    traits::samplers::{
        psrn::test::{assert_ordered_progression, kolmogorov_smirnov},
        PartialSample,
    },
};

use super::*;

#[test]
fn test_sample_gumbel_interval_progression() -> Fallible<()> {
    let mut sample = PartialSample::new(GumbelRV {
        shift: FBig::ZERO,
        scale: FBig::ONE,
    });
    assert_ordered_progression(&mut sample, 10);
    Ok(())
}

#[test]
fn test_gumbel_psrn() -> Fallible<()> {
    let gumbel = GumbelRV {
        shift: FBig::ZERO,
        scale: FBig::ONE,
    };

    let samples = (0..1000)
        .map(|_| PartialSample::new(gumbel.clone()).value::<f64>())
        .collect::<Fallible<Vec<f64>>>()?
        .try_into()
        .unwrap();

    kolmogorov_smirnov(samples, |x| (-(-x).exp()).exp())
}
