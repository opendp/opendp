use crate::{
    error::Fallible,
    traits::samplers::{
        psrn::test::{assert_ordered_progression, kolmogorov_smirnov},
        PartialSample,
    },
};

use super::*;

#[test]
fn test_sample_exponential_interval_progression() -> Fallible<()> {
    let (shift, scale) = (FBig::ZERO, FBig::ONE);
    let mut exp = PartialSample::new(ExponentialRV::new(shift, scale)?);
    let (l, r) = assert_ordered_progression(&mut exp, 20);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", exp.refinements);
    Ok(())
}

#[test]
fn test_exponential_psrn() -> Fallible<()> {
    let gumbel = ExponentialRV::new(FBig::ZERO, FBig::ONE)?;

    let samples = (0..1000)
        .map(|_| PartialSample::new(gumbel.clone()).value::<f64>())
        .collect::<Fallible<Vec<f64>>>()?
        .try_into()
        .unwrap();

    kolmogorov_smirnov(samples, |x| 1. - (-x).exp())
}
