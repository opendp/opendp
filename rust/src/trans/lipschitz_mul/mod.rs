use num::One;

use crate::{
    core::{Domain, Function, Metric, StabilityMap, Transformation},
    metrics::{AbsoluteDistance, LpDistance},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    traits::{
        AlertingAbs, CheckNull, ExactIntCast, FloatBits, InfAdd, InfMul, InfPow, InfSub,
        SaturatingMul, TotalOrd,
    },
};

use super::Float;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_lipschitz_float_mul<D, M>(
    constant: D::Atom,
    bounds: (D::Atom, D::Atom),
) -> Fallible<Transformation<D, D, M, M>>
where
    D: LipschitzMulFloatDomain,
    M: LipschitzMulFloatMetric<Distance = D::Atom>,
{
    let mantissa_bits = D::Atom::exact_int_cast(D::Atom::MANTISSA_BITS)?;
    let exponent_bias = D::Atom::exact_int_cast(D::Atom::EXPONENT_BIAS)?;
    let _2 = D::Atom::one() + D::Atom::one();
    let (lower, upper) = bounds;

    // Float arithmetic is computed with effectively infinite precision, and rounded to the nearest float
    // Consider * to be the float multiplication operation
    // d_out =  max_{v~v'} |f(v) - f(v')|
    //       <= max_{v~v'} |v * c - v' * c| where * is the float multiplication operation
    //       =  max_{v~v'} |(vc + e) - (v'c + e')| where e is the float error
    //       <= max_{v~v'} |(v - v')c + 2e|
    //       <= max_{v~v'} |v - v'||c| + 2|e| by triangle inequality
    //       =  d_in|c| + 2 max_{v~v'} |e|
    //       =  d_in|c| + 2 (ulp(w) / 2) where w = max(|L|, U).inf_mul(|c|), the greatest magnitude output
    //       =  d_in|c| + ulp(w)
    //       =  d_in|c| + 2^(exp - k) where exp = exponent(w) and k is the number of bits in the mantissa

    // max(|L|, U)
    let input_mag = lower.alerting_abs()?.total_max(upper)?;

    // w = max(|L|, U) * |c|
    let output_mag = input_mag.inf_mul(&constant.alerting_abs()?)?;

    // retrieve the unbiased exponent from the largest output magnitude float w
    let max_exponent = D::Atom::exact_int_cast(output_mag.exponent())?;
    let max_unbiased_exponent = max_exponent.neg_inf_sub(&exponent_bias)?;

    // greatest possible error is the ulp of the greatest possible output
    let output_ulp = _2.inf_pow(&max_unbiased_exponent.inf_sub(&mantissa_bits)?)?;

    Ok(Transformation::new(
        D::default(),
        D::default(),
        Function::new_fallible(move |arg: &D::Carrier| D::transform(&constant, &bounds, arg)),
        M::default(),
        M::default(),
        StabilityMap::new_fallible(move |d_in| {
            constant.alerting_abs()?.inf_mul(d_in)?.inf_add(&output_ulp)
        }),
    ))
}

/// Implemented for any domain that supports multiplication lipschitz extensions
pub trait LipschitzMulFloatDomain: Domain + Default {
    type Atom: 'static + Float;
    fn transform(
        constant: &Self::Atom,
        bounds: &(Self::Atom, Self::Atom),
        v: &Self::Carrier,
    ) -> Fallible<Self::Carrier>;
}

impl<T> LipschitzMulFloatDomain for AllDomain<T>
where
    T: 'static + Float,
{
    type Atom = T;
    fn transform(constant: &T, &(lower, upper): &(T, T), v: &T) -> Fallible<T> {
        Ok(constant.total_clamp(lower, upper)?.saturating_mul(v))
    }
}

impl<D: LipschitzMulFloatDomain> LipschitzMulFloatDomain for VectorDomain<D>
where
    D::Atom: Copy + SaturatingMul + CheckNull + TotalOrd,
{
    type Atom = D::Atom;
    fn transform(
        constant: &D::Atom,
        bounds: &(Self::Atom, Self::Atom),
        v: &Vec<D::Carrier>,
    ) -> Fallible<Vec<D::Carrier>> {
        v.iter()
            .map(|v_i| D::transform(constant, bounds, v_i))
            .collect()
    }
}

/// Implemented for any metric that supports multiplication lipschitz extensions
pub trait LipschitzMulFloatMetric: Metric {}

impl<T, const P: usize> LipschitzMulFloatMetric for LpDistance<T, P> {}
impl<T> LipschitzMulFloatMetric for AbsoluteDistance<T> {}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_lipschitz_mul() -> Fallible<()> {
        let extension =
            make_lipschitz_float_mul::<AllDomain<f64>, AbsoluteDistance<f64>>(2., (0., 10.))?;
        assert_eq!(extension.invoke(&1.3)?, 2.6);
        println!("{:?}", extension.invoke(&1.3));
        Ok(())
    }
}
