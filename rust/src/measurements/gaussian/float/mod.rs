use crate::{
    core::{Function, Measurement},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::get_discretization_consts,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{
        samplers::{sample_discrete_gaussian_Z2k, CastInternalRational},
        ExactIntCast, Float, FloatBits,
    },
};

use super::GaussianMeasure;

/// Make a Measurement that adds noise from the gaussian(`scale`) distribution to a scalar-valued float input.
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
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
pub fn make_scalar_float_gaussian<MO, T>(
    input_domain: AtomDomain<T>,
    input_metric: AbsoluteDistance<T>,
    scale: T,
    k: Option<i32>,
) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<T>, MO>>
where
    T: Float + CastInternalRational,
    MO: GaussianMeasure<AbsoluteDistance<T>, Atom = T>,
    i32: ExactIntCast<<T as FloatBits>::Bits>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let (k, relaxation) = get_discretization_consts(k)?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &T| sample_discrete_gaussian_Z2k(*arg, scale, k)),
        input_metric,
        MO::default(),
        MO::new_forward_map(scale, relaxation)?,
    )
}

/// Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
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
/// * `MO` - Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
pub fn make_vector_float_gaussian<MO, T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L2Distance<T>,
    scale: T,
    k: Option<i32>,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L2Distance<T>, MO>>
where
    T: Float + CastInternalRational,
    MO: GaussianMeasure<L2Distance<T>, Atom = T>,
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
                .map(|shift| sample_discrete_gaussian_Z2k(shift.clone(), scale, k))
                .collect()
        }),
        input_metric,
        MO::default(),
        MO::new_forward_map(scale, relaxation)?,
    )
}

#[cfg(test)]
mod tests {
    use crate::measures::ZeroConcentratedDivergence;

    use super::*;

    #[test]
    fn test_make_gaussian_mechanism() -> Fallible<()> {
        let measurement = make_scalar_float_gaussian::<ZeroConcentratedDivergence<_>, _>(
            AtomDomain::default(),
            AbsoluteDistance::default(),
            1.0f64,
            None,
        )?;
        let arg = 0.0;
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&0.1, &0.0050000001)?);
        Ok(())
    }

    #[test]
    fn test_make_gaussian_vec_mechanism() -> Fallible<()> {
        let measurement = make_vector_float_gaussian::<ZeroConcentratedDivergence<_>, _>(
            VectorDomain::new(AtomDomain::default()),
            L2Distance::default(),
            1.0f64,
            None,
        )?;
        let arg = vec![0.0, 1.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.map(&0.1)? <= 0.0050000001);
        Ok(())
    }
}
