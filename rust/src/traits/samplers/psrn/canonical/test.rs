use dashu::float::{
    FBig,
    round::mode::{Down, Up},
};
use num::FromPrimitive;

use crate::{
    error::Fallible,
    traits::samplers::{
        PartialSample, psrn::test::assert_ordered_progression, test::check_kolmogorov_smirnov,
    },
};

use super::*;

fn approximate_tradeoff(
    epsilon: f64,
    delta: f64,
) -> Fallible<(impl Fn(RBig) -> Fallible<RBig>, RBig)> {
    let epsilon = FBig::<Down>::try_from(epsilon)?;
    let delta = RBig::try_from(delta)?;
    let exp_epsilon = RBig::try_from(epsilon.clone().exp())?;
    let exp_neg_epsilon = RBig::try_from((-epsilon).with_rounding::<Up>().exp())?;
    let fixed_point = (RBig::ONE.clone() - &delta) / (RBig::ONE.clone() + &exp_epsilon);

    let tradeoff = move |alpha: RBig| {
        let first = RBig::ONE.clone() - &delta - &exp_epsilon * &alpha;
        let second = &exp_neg_epsilon * (RBig::ONE.clone() - &delta - alpha);
        Ok(first.max(second).max(RBig::ZERO.clone()))
    };
    Ok((tradeoff, fixed_point))
}

#[test]
fn test_sample_cnd_interval_progression() -> Fallible<()> {
    let (tradeoff, fixed_point) = approximate_tradeoff(1.0, 1e-6)?;
    let scale = &RBig::ONE;
    let mut cnd = PartialSample::new(CanonicalRV {
        shift: RBig::ZERO,
        scale: &scale,
        tradeoff: &tradeoff,
        fixed_point: &fixed_point,
    });
    let (l, r) = assert_ordered_progression(&mut cnd, 20)?;
    let (l, r) = (l.to_f64().value(), r.to_f64().value());
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
    let (tradeoff, fixed_point) = approximate_tradeoff(1.0, 1e-6)?;
    let scale = &RBig::ONE;
    let cnd = CanonicalRV {
        shift: RBig::ZERO,
        scale: &scale,
        tradeoff: &tradeoff,
        fixed_point: &fixed_point,
    };
    let samples = (0..5000)
        .map(|_| PartialSample::new(cnd.clone()).value())
        .collect::<Fallible<Vec<f64>>>()?;

    let samples = <[f64; 5000]>::try_from(samples).unwrap();

    let f_tradeoff = |x: f64| -> f64 {
        tradeoff(RBig::from_f64(x).unwrap())
            .unwrap()
            .to_f64()
            .value()
    };
    let f_c = fixed_point.to_f64().value();
    check_kolmogorov_smirnov(samples, |x| F_f(x, f_tradeoff, f_c))?;
    Ok(())
}
