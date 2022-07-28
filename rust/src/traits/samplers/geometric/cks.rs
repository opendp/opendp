use num::{One, Zero};

use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::AllDomain;
use crate::error::*;
use crate::measures::{MaxDivergence, ZeroConcentratedDivergence};
use crate::metrics::{AbsoluteDistance, L2Distance};
use crate::traits::samplers::{SampleBernoulli, SampleRademacher, SampleUniformInt};

use rug::{Complete, Integer, Rational};
pub fn make_base_discrete_laplace(
    scale: Rational,
) -> Fallible<
    Measurement<
        AllDomain<Integer>,
        AllDomain<Integer>,
        AbsoluteDistance<Rational>,
        MaxDivergence<Rational>,
    >,
> {
    if scale < 0 {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(enclose!(scale, move |arg: &Integer| {
            sample_discrete_laplace(scale.clone()).map(|noise| noise + arg)
        })),
        AbsoluteDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new(move |arg: &Rational| (arg / &scale).complete()),
    ))
}

pub fn make_base_discrete_gaussian(
    scale: Rational,
) -> Fallible<
    Measurement<
        AllDomain<Integer>,
        AllDomain<Integer>,
        L2Distance<Rational>,
        ZeroConcentratedDivergence<Rational>,
    >,
> {
    if scale < 0 {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(enclose!(scale, move |arg: &Integer| {
            sample_discrete_gaussian(scale.clone()).map(|noise| noise + arg)
        })),
        L2Distance::default(),
        ZeroConcentratedDivergence::default(),
        // TODO: fix this
        PrivacyMap::new(move |arg: &Rational| (arg / &scale).complete()),
    ))
}

impl SampleBernoulli<Rational> for bool {
    fn sample_bernoulli(prob: Rational, _constant_time: bool) -> Fallible<bool> {
        let (numer, denom) = prob.into_numer_denom();
        Integer::sample_uniform_int_0_u(denom).map(|s| s < numer)
    }
}

// sample from a Bernoulli(exp(-x)) distribution
// assumes x is a rational number in [0,1]
fn sample_bernoulli_exp1(v: Rational) -> Fallible<bool> {
    let mut k = Integer::one();
    loop {
        if bool::sample_bernoulli(v.clone() / &k, false)? {
            k += 1;
        } else {
            return Ok(k.is_odd());
        }
    }
}

// sample from a Bernoulli(exp(-x)) distribution
// assumes x is a rational number >=0
fn sample_bernoulli_exp(mut v: Rational) -> Fallible<bool> {
    // Sample floor(x) independent Bernoulli(exp(-1))
    // If all are 1, return Bernoulli(exp(-(x-floor(x))))
    while v > 1 {
        if sample_bernoulli_exp1(1.into())? {
            v -= 1;
        } else {
            return Ok(false);
        }
    }
    sample_bernoulli_exp1(v)
}

// sample from a geometric(1-exp(-x)) distribution
// assumes x is a rational number >= 0
fn sample_geometric_exp_slow(v: Rational) -> Fallible<Integer> {
    if v < 0 {
        return fallible!(FailedFunction, "v must be non-negative");
    }
    let mut k = 0.into();
    loop {
        if sample_bernoulli_exp(v.clone())? {
            k += 1;
        } else {
            return Ok(k);
        }
    }
}

// sample from a geometric(1-exp(-x)) distribution
// assumes x >= 0 rational
fn sample_geometric_exp_fast(v: Rational) -> Fallible<Integer> {
    if v < 0 {
        return fallible!(FailedFunction, "v must be non-negative");
    }
    if v.is_zero() {
        return Ok(0.into());
    }

    let (numer, denom) = v.into_numer_denom();
    let mut u = Integer::sample_uniform_int_0_u(denom.clone())?;
    while !sample_bernoulli_exp(Rational::from((u.clone(), denom.clone())))? {
        u = Integer::sample_uniform_int_0_u(denom.clone())?;
    }
    let v2 = sample_geometric_exp_slow(Rational::one())?;
    Ok((v2 * denom + u) / numer)
}

fn sample_discrete_laplace(scale: Rational) -> Fallible<Integer> {
    loop {
        let sign = Integer::sample_standard_rademacher()?;
        let magnitude = sample_geometric_exp_fast(scale.clone().recip())?;
        if !(sign.is_one() && magnitude.is_zero()) {
            return Ok(sign * magnitude);
        }
    }
}

fn sample_discrete_gaussian(scale: Rational) -> Fallible<Integer> {
    let t = scale.clone().floor() + 1i8;
    let sigma2 = scale.square();
    loop {
        let candidate = sample_discrete_laplace(t.clone())?;
        let x = candidate.clone().abs() - sigma2.clone() / &t;
        let bias = x.square() / (2 * sigma2.clone());
        if sample_bernoulli_exp(bias)? {
            return Ok(candidate);
        }
    }
}

// fn sqrt_floor(x: Rational) -> Integer {
//     let mut a = Integer::zero(); // maintain a^2<=x
//     let mut b = Integer::one(); // maintain b^2>x

//     // double to get upper bound
//     while b.clone().square() <= x {
//         b *= 2;
//     }

//     // now do binary search
//     // c=floor((a+b)/2)
//     while a.clone() + 1 < b {
//         let c = (a.clone() + b.clone()) / 2i8;
//         if c.clone().square() <= x {
//             a = c;
//         } else {
//             b = c;
//         }
//     }
//     a
// }

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn test_make_discrete_laplace_mechanism() -> Fallible<()> {
        let meas = make_base_discrete_laplace(10.0.try_into().unwrap())?;

        println!("res {:?}", meas.invoke(&0.into())?);
        println!("map {:?}", meas.map(&0.1.try_into().unwrap())?.to_f64());
        Ok(())
    }

    #[test]
    fn test_make_discrete_gaussian_mechanism() -> Fallible<()> {
        let meas = make_base_discrete_gaussian(10.0.try_into().unwrap())?;

        println!("res {:?}", meas.invoke(&0.into())?);
        println!("map {:?}", meas.map(&0.1.try_into().unwrap())?.to_f64());
        Ok(())
    }
}
