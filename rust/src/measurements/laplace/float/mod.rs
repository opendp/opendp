use crate::core::{Function, Measurement, PrivacyMap};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::measures::MaxDivergence;
use crate::metrics::{AbsoluteDistance, L1Distance};
use crate::traits::samplers::{sample_discrete_laplace_Z2k, CastInternalRational};
use crate::traits::{ExactIntCast, Float, FloatBits};

use super::laplace_map;

/// Make a Measurement that adds noise from the Laplace(`scale`) distribution to a scalar value.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `k` - The noise granularity in terms of 2^k.
pub fn make_scalar_float_laplace<T>(
    input_domain: AtomDomain<T>,
    input_metric: AbsoluteDistance<T>,
    scale: T,
    k: Option<i32>,
) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<T>, MaxDivergence<T>>>
where
    T: Float + CastInternalRational,
    i32: ExactIntCast<<T as FloatBits>::Bits>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let (k, relaxation) = get_discretization_consts(k)?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |shift: &T| sample_discrete_laplace_Z2k(*shift, scale, k)),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(laplace_map(scale, relaxation)),
    )
}

/// Make a Measurement that adds noise from the Laplace(`scale`) distribution to a vector value.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `k` - The noise granularity in terms of 2^k.
pub fn make_vector_float_laplace<T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L1Distance<T>,
    scale: T,
    k: Option<i32>,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L1Distance<T>, MaxDivergence<T>>>
where
    T: Float + CastInternalRational,
    i32: ExactIntCast<<T as FloatBits>::Bits>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let (k, mut relaxation): (i32, T) = get_discretization_consts(k)?;

    if !relaxation.is_zero() {
        let size = input_domain.size.ok_or_else(|| {
            err!(
                MakeMeasurement,
                "domain size must be known if discretization is not exact"
            )
        })?;
        relaxation = relaxation.inf_mul(&T::inf_cast(size)?)?;
    }

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<T>| {
            arg.iter()
                .map(|shift| sample_discrete_laplace_Z2k(*shift, scale, k))
                .collect()
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(laplace_map(scale, relaxation)),
    )
}

// proof should show that the return is always a valid (k, relaxation) pairing
pub(crate) fn get_discretization_consts<T>(k: Option<i32>) -> Fallible<(i32, T)>
where
    T: Float,
    i32: ExactIntCast<T::Bits>,
{
    // the discretization may only be as fine as the subnormal ulp
    let k_min =
        -i32::exact_int_cast(T::EXPONENT_BIAS)? - i32::exact_int_cast(T::MANTISSA_BITS)? + 1;
    let k = k.unwrap_or(k_min).max(k_min);

    let _2 = T::exact_int_cast(2)?;

    // input has granularity 2^{k_min} (subnormal float precision)
    let input_gran = _2.neg_inf_powi(k_min.into())?;

    // discretization rounds to the nearest 2^k
    let output_gran = _2.inf_powi(k.into())?;

    // the worst-case increase in sensitivity due to discretization is
    //     the range, minus the smallest step in the range
    let relaxation = output_gran.inf_sub(&input_gran)?;

    Ok((k, relaxation))
}

#[cfg(all(test, feature = "partials"))]
mod test;
