use fixed::traits::Fixed;
use fixed::types::I16F16;
use num::{One, Zero};

use crate::core::{Function, Measurement, PrivacyRelation};
use crate::dist::{AbsoluteDistance, L2Distance, MaxDivergence};
use crate::dom::{AllDomain};
use crate::error::*;
use crate::samplers::{SampleBernoulli, SampleDiscreteUniform, SampleStandardRademacher};

pub fn make_base_discrete_laplace(
    scale: I16F16
) -> Fallible<Measurement<AllDomain<I16F16>, AllDomain<I16F16>, AbsoluteDistance<I16F16>, MaxDivergence<I16F16>>> {
    if scale.is_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(enclose!(scale, move |arg: &I16F16|
            I16F16::sample_discrete_laplace(scale, false).map(|noise| noise + arg))),
        AbsoluteDistance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale.recip()),
    ))
}

pub fn make_base_discrete_gaussian(
    scale: I16F16
) -> Fallible<Measurement<AllDomain<I16F16>, AllDomain<I16F16>, L2Distance<I16F16>, MaxDivergence<I16F16>>> {
    if scale.is_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(enclose!(scale, move |arg: &I16F16|
            I16F16::sample_discrete_gaussian(scale, false).map(|noise| noise + arg))),
        L2Distance::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale.recip()),
    ))
}

fn floorsqrt(x: I16F16) -> I16F16 {
    let mut a = 0;  // maintain a^2<=x
    let mut b = 1;  // #maintain b^2>x

    // double to get upper bound
    while b * b <= x { b *= 2; }

    // now do binary search
    // c=floor((a+b)/2)
    while a + 1 < b {
        let c = (a + b) / 2;
        if c * c <= x { a = c; } else { b = c; }
    }
    I16F16::from_bits(a)
}


trait Rational {
    type Int;
    fn numerator(&self) -> Self::Int;
    fn denominator(&self) -> Self::Int;
}

impl Rational for I16F16 {
    type Int = <I16F16 as Fixed>::Bits;

    fn numerator(&self) -> Self::Int {
        self.to_bits()
    }

    fn denominator(&self) -> Self::Int {
        (2 as Self::Int).pow(Self::FRAC_NBITS)
    }
}

impl SampleBernoulli<I16F16> for bool {
    fn sample_bernoulli(prob: I16F16, _constant_time: bool) -> Fallible<Self> {
        type Int = <I16F16 as Rational>::Int;
        let s_numer = Int::sample_uniform_range(Int::zero(), prob.numerator())?;
        let s_denom = Int::sample_uniform_range(Int::zero(), prob.denominator())?;
        Ok(s_numer > s_denom)
    }
}

trait SampleDiscreteGaussian: Fixed
    where bool: SampleBernoulli<Self> {
    fn sample_bernoulli_exp1(v: Self, constant_time: bool) -> Fallible<bool>;
    fn sample_bernoulli_exp(v: Self, constant_time: bool) -> Fallible<bool>;
    fn sample_geometric_exp_slow(v: Self, constant_time: bool) -> Fallible<Self::Bits>;
    fn sample_geometric_exp_fast(v: Self, constant_time: bool) -> Fallible<Self::Bits>;
    fn sample_discrete_laplace(scale: Self, constant_time: bool) -> Fallible<Self>;
    fn sample_discrete_gaussian(scale: Self, constant_time: bool) -> Fallible<Self>;
}

impl SampleDiscreteGaussian for I16F16 {
    // sample from a Bernoulli(exp(-x)) distribution
    // assumes x is a rational number in [0,1]
    fn sample_bernoulli_exp1(v: Self, constant_time: bool) -> Fallible<bool> {
        if constant_time { return fallible!(FailedFunction, "constant time execution is not implemented") }
        let mut k = Self::one();
        loop {
            if bool::sample_bernoulli(v / k, constant_time)? {
                k += Self::one();
            } else {
                return Ok((k % 2).is_zero())
            }
        }
    }

    // sample from a Bernoulli(exp(-x)) distribution
    // assumes x is a rational number >=0
    fn sample_bernoulli_exp(mut v: Self, constant_time: bool) -> Fallible<bool> {
        if constant_time { return fallible!(FailedFunction, "constant time execution is not implemented") }
        // Sample floor(x) independent Bernoulli(exp(-1))
        // If all are 1, return Bernoulli(exp(-(x-floor(x))))
        while v > Self::one() {
            if Self::sample_bernoulli_exp1(Self::one(), constant_time)? {
                v -= Self::one();
            } else {
                return Ok(false)
            }
        }
        Self::sample_bernoulli_exp1(v, constant_time)
    }

    // sample from a geometric(1-exp(-x)) distribution
    // assumes x is a rational number >= 0
    fn sample_geometric_exp_slow(v: Self, constant_time: bool) -> Fallible<Self::Bits> {
        if v.is_negative() { return fallible!(FailedFunction, "v must be non-negative") }
        let mut k = Self::Bits::zero();
        loop {
            if Self::sample_bernoulli_exp(v, constant_time)? {
                k += Self::Bits::one();
            } else {
                return Ok(k)
            }
        }
    }

    // sample from a geometric(1-exp(-x)) distribution
    // assumes x >= 0 rational
    fn sample_geometric_exp_fast(v: Self, constant_time: bool) -> Fallible<Self::Bits> {
        if constant_time { return fallible!(FailedFunction, "constant time execution is not implemented") }
        if v.is_negative() { return fallible!(FailedFunction, "v must be non-negative") }
        if v.is_zero() { return Ok(Self::Bits::zero()) }

        let denom = v.denominator();
        let mut u = Self::Bits::sample_uniform_range(Self::Bits::zero(), denom)?;
        while !Self::sample_bernoulli_exp(I16F16::from_bits(u), constant_time)? {
            u = Self::Bits::sample_uniform_range(Self::Bits::zero(), denom)?;
        }
        let v2 = Self::sample_geometric_exp_slow(Self::one(), constant_time)?;
        Ok((v2 * denom + u) / v.numerator())
    }

    fn sample_discrete_laplace(scale: Self, constant_time: bool) -> Fallible<Self> {
        loop {
            let sign = I16F16::sample_standard_rademacher()?;
            let magnitude = I16F16::sample_geometric_exp_fast(scale.recip(), constant_time)?;
            if !(sign.is_one() && magnitude.is_zero()) {
                return Ok(sign * magnitude)
            }
        }
    }

    fn sample_discrete_gaussian(scale: Self, constant_time: bool) -> Fallible<Self> {
        let t = floorsqrt(scale) + I16F16::one();
        loop {
            let candidate = Self::sample_discrete_laplace(scale, constant_time)?;
            let x = candidate.abs() - scale / t;
            let bias = (x * x) / (2 * scale);
            if Self::sample_bernoulli_exp(bias, constant_time)? {
                return Ok(candidate);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_gaussian(fixed!(1.0: I16F16))?;
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_gaussian(fixed!(1.0: I16F16))?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.function.eval(&arg)?;

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001))?);
        Ok(())
    }
}
