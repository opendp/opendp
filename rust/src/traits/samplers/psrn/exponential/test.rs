use crate::{
    error::Fallible,
    traits::samplers::{
        PartialSample, psrn::test::assert_ordered_progression, test::check_kolmogorov_smirnov,
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
    let rv = ExponentialRV::new(FBig::ZERO, FBig::ONE)?;

    let samples = (0..1000)
        .map(|_| PartialSample::new(rv.clone()).value::<f64>())
        .collect::<Fallible<Vec<f64>>>()?
        .try_into()
        .unwrap();

    check_kolmogorov_smirnov(samples, |x| 1. - (-x).exp())
}

#[test]
fn test_exponential_psrn_zero() -> Fallible<()> {
    let rv = ExponentialRV {
        shift: FBig::ZERO,
        scale: FBig::ZERO,
    };
    assert_eq!(PartialSample::new(rv).value::<f64>()?, 0.0);
    Ok(())
}
