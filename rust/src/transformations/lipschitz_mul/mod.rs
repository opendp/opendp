use dashu::integer::IBig;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::AtomDomain,
    error::Fallible,
    metrics::AbsoluteDistance,
    traits::Float,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(constant(c_type = "void *")),
    generics(TA(suppress)),
    derived_types(TA = "$get_atom(get_carrier_type(input_domain))")
)]
/// Make a transformation that multiplies an aggregate by a constant.
///
/// The bounds clamp the input, in order to bound the increase in sensitivity from float rounding.
///
/// # Arguments
/// * `input_domain` - The domain of the input.
/// * `input_metric` - The metric of the input.
/// * `constant` - The constant to multiply aggregates by.
/// * `bounds` - Tuple of inclusive lower and upper bounds.
///
/// # Generics
/// * `TA` - Atomic type.
pub fn make_lipschitz_float_mul<TA: Float>(
    input_domain: AtomDomain<TA>,
    input_metric: AbsoluteDistance<TA>,
    constant: TA,
    bounds: (TA, TA),
) -> Fallible<
    Transformation<AtomDomain<TA>, AbsoluteDistance<TA>, AtomDomain<TA>, AbsoluteDistance<TA>>,
> {
    if input_domain.nan() {
        return fallible!(MakeTransformation, "input_domain may not contain NaN.");
    }

    let _2 = TA::exact_int_cast(2)?;
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
    let max_unbiased_exponent = max_exponent - TA::EXPONENT_BIAS.into();

    // greatest possible error is the ulp of the greatest possible output
    let output_ulp = _2.inf_powi(max_unbiased_exponent - TA::MANTISSA_BITS.into())?;

    Transformation::new(
        input_domain,
        input_metric.clone(),
        AtomDomain::new_non_nan(),
        input_metric,
        Function::new_fallible(move |arg: &TA| {
            Ok(arg.total_clamp(lower, upper)?.saturating_mul(&constant))
        }),
        StabilityMap::new_fallible(move |d_in| {
            constant.alerting_abs()?.inf_mul(d_in)?.inf_add(&output_ulp)
        }),
    )
}
