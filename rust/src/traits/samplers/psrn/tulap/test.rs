use crate::traits::samplers::{pinpoint, psrn::test::test_progression};

use super::*;

#[test]
fn test_sample_tulap_interval_progression() -> Fallible<()> {
    let mut tulap = TulapPSRN::new(
        RBig::ZERO,
        FBig::ONE.with_precision(50).value(),
        RBig::try_from(1e-6).unwrap(),
    )?;
    let (l, r) = test_progression(&mut tulap, 20);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", tulap.refinements());
    Ok(())
}

#[test]
fn test_tulap_psrn() -> Fallible<()> {
    let samples = (0..1000)
        .map(|_| {
            pinpoint(&mut TulapPSRN::new(
                RBig::ZERO,
                FBig::ONE.with_precision(50).value(),
                RBig::try_from(1e-6).unwrap(),
            )?)
        })
        .collect::<Fallible<Vec<f64>>>()?;
    println!("{:?}", samples);
    Ok(())
}
