use dashu::{
    float::round::mode::{Down, Up},
    rbig,
};

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
    let mut exp = PartialSample::new(ExponentialRV { shift, scale });
    let (l, r) = assert_ordered_progression(&mut exp, 400);
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
    println!("{l:?}, {r:?}, {}", exp.refinements);
    Ok(())
}

#[test]
fn test_exponential_psrn() -> Fallible<()> {
    let (shift, scale) = (FBig::ZERO, FBig::ONE);
    let rv = ExponentialRV { shift, scale };

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

#[test]
fn test_exponential_inverse_cdf() -> Fallible<()> {
    fn f_unif_comp_inv<R: ODPRound>(f_unif_comp: FBig) -> FBig {
        (-f_unif_comp.with_rounding::<R::C>().ln()).with_rounding()
    }
    let f_unif_comp = FBig::try_from(0.5).unwrap();
    assert!(
        f_unif_comp_inv::<Up>(f_unif_comp.clone()) > f_unif_comp_inv::<Down>(f_unif_comp.clone())
    );

    fn f_unif_comp_inv_bad<R: ODPRound>(f_unif_comp: FBig) -> FBig {
        (-f_unif_comp.with_rounding::<R>().ln()).with_rounding()
    }
    // when rounding mode is not reversed, the output rounds in the wrong direction
    assert!(
        f_unif_comp_inv_bad::<Up>(f_unif_comp.clone()) < f_unif_comp_inv_bad::<Down>(f_unif_comp)
    );

    // directly test the inverse cdf
    let exp = ExponentialRV {
        shift: FBig::ZERO,
        scale: FBig::ONE,
    };
    let r_unif_comp = rbig!(1 / 3);
    assert!(
        exp.inverse_cdf::<Up>(r_unif_comp.clone(), 3).unwrap()
            > exp.inverse_cdf::<Down>(r_unif_comp, 3).unwrap()
    );
    Ok(())
}
