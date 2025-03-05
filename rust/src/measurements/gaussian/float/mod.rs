use crate::{
    core::{Function, Measurement},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::get_discretization_consts,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{
        samplers::sample_discrete_gaussian_Z2k, CastInternalRational, ExactIntCast, Float,
        FloatBits, InfCast,
    },
};

use super::GaussianMeasure;

/// Make a Measurement that adds noise from the Gaussian(`scale`) distribution to a scalar-valued float input.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `input_metric` - Metric of the data type to be privatized. Valid values are `AbsoluteDistance<T>` or `L2Distance<T>`.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `D` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
pub fn make_scalar_float_gaussian<MO, T>(
    input_domain: AtomDomain<T>,
    input_metric: AbsoluteDistance<T>,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<T>, MO>>
where
    T: Float + CastInternalRational,
    MO: GaussianMeasure<AbsoluteDistance<T>>,
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    f64: InfCast<T>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({:?}) must not be negative", scale);
    }

    let (k, relaxation) = get_discretization_consts::<T>(k)?;

    let relaxation = f64::inf_cast(relaxation)?;
    let r_scale = scale.into_rational()?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &T| {
            let sample = sample_discrete_gaussian_Z2k(arg.into_rational()?, r_scale.clone(), k)?;

            // postprocessing: round to nearest T
            Ok(T::from_rational(sample))
        }),
        input_metric,
        MO::default(),
        MO::new_forward_map(scale, relaxation),
    )
}

/// Make a Measurement that adds noise from the Gaussian(`scale`) distribution to the vector-valued input.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `input_metric` - Metric of the data type to be privatized. Valid values are `AbsoluteDistance<T>` or `L2Distance<T>`.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `D` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
pub fn make_vector_float_gaussian<MO, T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L2Distance<T>,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L2Distance<T>, MO>>
where
    T: Float + CastInternalRational,
    MO: GaussianMeasure<L2Distance<T>>,
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    f64: InfCast<T>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({:?}) must not be negative", scale);
    }

    let (k, mut relaxation) = get_discretization_consts::<T>(k)?;

    if !relaxation.is_zero() {
        let size = input_domain.size.ok_or_else(|| {
            err!(
                MakeMeasurement,
                "domain size must be known if discretization is not exact"
            )
        })?;
        relaxation = relaxation.inf_mul(&T::inf_cast(size)?)?;
    }

    let relaxation = f64::inf_cast(relaxation)?;
    let r_scale = scale.into_rational()?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<T>| {
            arg.iter()
                .map(|shift| {
                    let sample =
                        sample_discrete_gaussian_Z2k(shift.into_rational()?, r_scale.clone(), k)?;

                    // postprocessing: round to nearest T
                    Ok(T::from_rational(sample))
                })
                .collect()
        }),
        input_metric,
        MO::default(),
        MO::new_forward_map(scale, relaxation),
    )
}

#[cfg(test)]
mod test;
