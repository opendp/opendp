use dashu::integer::IBig;
use num::One;
use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, LpDistance},
    traits::{
        AlertingAbs, CheckNull, Float, FloatBits, InfAdd, InfMul, InfPowI, ProductOrd,
        SaturatingMul,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
        constant(rust_type = "T", c_type = "void *"),
        bounds(rust_type = "(T, T)")
    ),
    generics(
        D(default = "AtomDomain<T>", generics = "T"),
        M(default = "AbsoluteDistance<T>", generics = "T")
    ),
    derived_types(T = "$get_atom_or_infer(D, constant)")
)]
/// Make a transformation that multiplies an aggregate by a constant.
///
/// The bounds clamp the input, in order to bound the increase in sensitivity from float rounding.
///
/// # Arguments
/// * `constant` - The constant to multiply aggregates by.
/// * `bounds` - Tuple of inclusive lower and upper bounds.
///
/// # Generics
/// * `D` - Domain of the function. Must be `AtomDomain<T>` or `VectorDomain<AtomDomain<T>>`
/// * `M` - Metric. Must be `AbsoluteDistance<T>`, `L1Distance<T>` or `L2Distance<T>`
pub fn make_lipschitz_float_mul<D, M>(
    constant: D::Atom,
    bounds: (D::Atom, D::Atom),
) -> Fallible<Transformation<D, D, M, M>>
where
    D: LipschitzMulFloatDomain,
    M: LipschitzMulFloatMetric<Distance = D::Atom>,
    (D, M): MetricSpace,
{
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
    let max_exponent: IBig = output_mag.raw_exponent().into();
    let max_unbiased_exponent = max_exponent - D::Atom::EXPONENT_BIAS.into();

    // greatest possible error is the ulp of the greatest possible output
    let output_ulp = _2.inf_powi(max_unbiased_exponent - D::Atom::MANTISSA_BITS.into())?;

    Transformation::new(
        D::default(),
        D::default(),
        Function::new_fallible(move |arg: &D::Carrier| D::transform(&constant, &bounds, arg)),
        M::default(),
        M::default(),
        StabilityMap::new_fallible(move |d_in| {
            constant.alerting_abs()?.inf_mul(d_in)?.inf_add(&output_ulp)
        }),
    )
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

impl<T> LipschitzMulFloatDomain for AtomDomain<T>
where
    T: 'static + Float,
{
    type Atom = T;
    fn transform(constant: &T, &(lower, upper): &(T, T), v: &T) -> Fallible<T> {
        Ok(v.total_clamp(lower, upper)?.saturating_mul(constant))
    }
}

impl<D: LipschitzMulFloatDomain> LipschitzMulFloatDomain for VectorDomain<D>
where
    D::Atom: Copy + SaturatingMul + CheckNull + ProductOrd,
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

impl<const P: usize, Q> LipschitzMulFloatMetric for LpDistance<P, Q> {}
impl<Q> LipschitzMulFloatMetric for AbsoluteDistance<Q> {}

#[cfg(test)]
mod test;
