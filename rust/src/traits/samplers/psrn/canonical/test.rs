use num::FromPrimitive;

use crate::{
    error::Fallible,
    measurements::approximate_to_tradeoff,
    traits::{
        CastInternalRational,
        samplers::{
            PartialSample, psrn::test::assert_ordered_progression, test::check_kolmogorov_smirnov,
        },
    },
};

use super::*;

#[test]
fn test_sample_cnd_interval_progression() -> Fallible<()> {
    let (tradeoff, c) = approximate_to_tradeoff((1.0, 1e-6))?;
    let scale = &RBig::ONE;
    let mut cnd = PartialSample::new(CanonicalRV {
        shift: RBig::ZERO,
        scale: &scale,
        tradeoff: &tradeoff,
        fixed_point: &c,
    });
    let (l, r) = assert_ordered_progression(&mut cnd, 20);
    let (l, r) = (f64::from_rational(l), f64::from_rational(r));
    println!("{l:?}, {r:?}, {}", cnd.refinements);
    Ok(())
}

// CDF from Definition 3.7 in https://arxiv.org/pdf/2108.04303
#[allow(non_snake_case)]
fn F_f(x: f64, f: impl Fn(f64) -> f64 + Clone, c: f64) -> f64 {
    if x < -0.5 {
        f(1. - F_f(x + 1.0, f.clone(), c))
    } else if x > 0.5 {
        1.0 - f(F_f(x - 1.0, f.clone(), c))
    } else {
        c * (0.5 - x) + (1.0 - c) * (x + 0.5)
    }
}

#[test]
fn test_cnd_psrn() -> Fallible<()> {
    let (tradeoff, c) = approximate_to_tradeoff((1.0, 1e-6))?;
    let scale = &RBig::ONE;
    let cnd = CanonicalRV {
        shift: RBig::ZERO,
        scale: &scale,
        tradeoff: &tradeoff,
        fixed_point: &c,
    };
    let samples = (0..5000)
        .map(|_| PartialSample::new(cnd.clone()).value())
        .collect::<Fallible<Vec<f64>>>()?;

    let samples = <[f64; 5000]>::try_from(samples).unwrap();

    let f_tradeoff = |x: f64| -> f64 { f64::from_rational(tradeoff(RBig::from_f64(x).unwrap())) };
    let f_c = f64::from_rational(c);
    check_kolmogorov_smirnov(samples, |x| F_f(x, f_tradeoff, f_c))?;
    Ok(())
}
