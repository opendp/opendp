use crate::{
    error::Fallible,
    measurements::approximate_to_tradeoff,
    traits::samplers::{PartialSample, psrn::test::assert_ordered_progression},
};

use super::*;

#[test]
fn test_sample_tulap_interval_progression() -> Fallible<()> {
    let (tradeoff, c) = approximate_to_tradeoff((1.0, 1e-6))?;
    let scale = &RBig::ONE;
    let mut tulap = PartialSample::new(CanonicalRV {
        shift: RBig::ZERO,
        scale: &scale,
        tradeoff: &tradeoff,
        fixed_point: &c,
    });
    let (l, r) = assert_ordered_progression(&mut tulap, 20);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", tulap.refinements);
    Ok(())
}

#[test]
fn test_tulap_psrn() -> Fallible<()> {
    let (tradeoff, c) = approximate_to_tradeoff((1.0, 1e-6))?;
    let scale = &RBig::ONE;
    let tulap = CanonicalRV {
        shift: RBig::ZERO,
        scale: &scale,
        tradeoff: &tradeoff,
        fixed_point: &c,
    };
    let samples = (0..1000)
        .map(|_| PartialSample::new(tulap.clone()).value())
        .collect::<Fallible<Vec<f64>>>()?;
    println!("{:?}", samples);
    Ok(())
}
