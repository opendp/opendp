use crate::traits::samplers::{psrn::test::assert_ordered_progression, PartialSample};

use super::*;

#[test]
fn test_sample_tulap_interval_progression() -> Fallible<()> {
    let tulap_rv = TulapRV::new(
        RBig::ZERO,
        FBig::ONE.with_precision(50).value(),
        RBig::try_from(1e-6).unwrap(),
    )?;
    let mut tulap = PartialSample::new(tulap_rv);
    let (l, r) = assert_ordered_progression(&mut tulap, 20);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", tulap.refinements);
    Ok(())
}

#[test]
fn test_tulap_psrn() -> Fallible<()> {
    let samples = (0..1000)
        .map(|_| {
            PartialSample::new(TulapRV::new(
                RBig::ZERO,
                FBig::ONE.with_precision(50).value(),
                RBig::try_from(1e-6).unwrap(),
            )?)
            .value()
        })
        .collect::<Fallible<Vec<f64>>>()?;
    println!("{:?}", samples);
    Ok(())
}
