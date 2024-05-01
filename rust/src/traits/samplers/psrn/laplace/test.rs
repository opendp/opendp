use crate::{
    error::Fallible,
    traits::samplers::{
        psrn::test::{assert_ordered_progression, kolmogorov_smirnov},
        PartialSample,
    },
};

use super::*;

#[test]
fn test_sample_laplace_interval_progression() -> Fallible<()> {
    let mut laplace = PartialSample::new(LaplaceRV {
        shift: FBig::ZERO,
        scale: FBig::ONE,
    });
    let (l, r) = assert_ordered_progression(&mut laplace, 20);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", laplace.refinements);
    Ok(())
}

#[test]
fn test_laplace_psrn() -> Fallible<()> {
    let laplace = LaplaceRV {
        shift: FBig::ZERO,
        scale: FBig::ONE,
    };
    let samples: [f64; 1000] = (0..1000)
        .map(|_| PartialSample::new(laplace.clone()).value())
        .collect::<Fallible<Vec<f64>>>()?
        .try_into()
        .unwrap();
    kolmogorov_smirnov(samples, |x| {
        if x.is_sign_negative() {
            x.exp() / 2.0
        } else {
            1.0 - (-x).exp() / 2.0
        }
    })
}
