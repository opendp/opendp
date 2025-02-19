use std::convert::TryFrom;

use dashu::{integer::IBig, rational::RBig};
use num::Zero;

use crate::{
    core::{Function, Measurement},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, L2Distance},
    traits::{samplers::sample_discrete_gaussian, CheckAtom, SaturatingCast},
};

use super::GaussianMeasure;

/// Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
///
/// # Generics
/// * `T` - Type of input data.
/// * `MO` - Output measure. The only valid measure is `ZeroConcentratedDivergence`.
/// * `QI` - Input distance. The type of sensitivities. Can be any integer or float.
pub fn make_scalar_integer_gaussian<T, MO, QI>(
    input_domain: AtomDomain<T>,
    input_metric: AbsoluteDistance<QI>,
    scale: f64,
) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<QI>, MO>>
where
    T: CheckAtom + SaturatingCast<IBig>,
    IBig: From<T>,

    MO: GaussianMeasure<AbsoluteDistance<QI>>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
    }
    let scale_rational = RBig::try_from(scale).map_err(|_| {
        err!(
            MakeMeasurement,
            "scale ({}) is not representable as a fraction",
            scale
        )
    })?;

    Measurement::new(
        input_domain,
        if scale.is_zero() {
            Function::new(move |arg: &T| arg.clone())
        } else {
            Function::new_fallible(move |arg: &T| {
                // exact conversion to bignum int
                let arg = IBig::from(arg.clone());
                // exact sampling of noise
                let noise = sample_discrete_gaussian(scale_rational.clone())?;
                // exact addition, and then postprocess by casting to D::Atom
                //     clamp to the data type's bounds if out of range
                Ok(T::saturating_cast(arg + noise))
            })
        },
        input_metric,
        MO::default(),
        MO::new_forward_map(scale, 0.0),
    )
}

/// Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
///
/// # Generics
/// * `T` - Type of input data.
/// * `MO` - Output measure. The only valid measure is `ZeroConcentratedDivergence`.
/// * `QI` - Input distance. The type of sensitivities. Can be any integer or float.
pub fn make_vector_integer_gaussian<T, MO, QI>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L2Distance<QI>,
    scale: f64,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L2Distance<QI>, MO>>
where
    T: CheckAtom + SaturatingCast<IBig>,
    IBig: From<T>,

    MO: GaussianMeasure<L2Distance<QI>>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
    }
    let scale_rational = RBig::try_from(scale).map_err(|_| {
        err!(
            MakeMeasurement,
            "scale ({}) is not representable as a fraction",
            scale
        )
    })?;

    Measurement::new(
        input_domain,
        if scale.is_zero() {
            Function::new(move |arg: &Vec<T>| arg.clone())
        } else {
            Function::new_fallible(move |arg: &Vec<T>| {
                arg.iter()
                    .map(|v| {
                        // exact conversion to bignum int
                        let v = IBig::from(v.clone());
                        // exact sampling of noise
                        let noise = sample_discrete_gaussian(scale_rational.clone())?;
                        // exact addition, and then postprocess by casting to T
                        //     clamp to the data type's bounds if out of range
                        Ok(T::saturating_cast(v + noise))
                    })
                    .collect()
            })
        },
        input_metric,
        MO::default(),
        MO::new_forward_map(scale, 0.0),
    )
}

#[cfg(test)]
mod test;
