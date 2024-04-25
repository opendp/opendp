use std::convert::TryFrom;

use dashu::{integer::IBig, rational::RBig};

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::laplace::laplace_map,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{samplers::sample_discrete_laplace, Float, InfCast, Integer, SaturatingCast},
};

/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
/// using an efficient algorithm on rational bignums.
///
/// # Citations
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `input_domain` - The domain of integers of a finite data type.
/// * `input_metric` - The absolute distance metric.
/// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
///
/// # Generics
/// * `T` - Data type of input data
/// * `QO` - Data type of the output distance and scale.
pub fn make_scalar_integer_laplace<T, QO>(
    input_domain: AtomDomain<T>,
    input_metric: AbsoluteDistance<T>,
    scale: QO,
) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<T>, MaxDivergence<QO>>>
where
    T: Integer + SaturatingCast<IBig>,
    QO: Float + InfCast<T>,
    IBig: From<T>,
    RBig: TryFrom<QO>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let r_scale =
        RBig::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Measurement::new(
        input_domain,
        if scale.is_zero() {
            Function::new(move |x: &T| *x)
        } else {
            Function::new_fallible(move |x: &T| {
                let release = IBig::from(x.clone()) + sample_discrete_laplace(r_scale.clone())?;
                // postprocess
                Ok(T::saturating_cast(release))
            })
        },
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(laplace_map(scale, QO::zero())),
    )
}

/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
/// using an efficient algorithm on rational bignums.
///
/// # Citations
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `input_domain` - The domain of vectors of integers of a finite data type.
/// * `input_metric` - The L1 distance metric.
/// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
///
/// # Generics
/// * `T` - Data type of input data
/// * `QO` - Data type of the output distance and scale.
pub fn make_vector_integer_laplace<T, QO>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L1Distance<T>,
    scale: QO,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L1Distance<T>, MaxDivergence<QO>>>
where
    T: Integer + SaturatingCast<IBig>,
    QO: Float + InfCast<T>,
    IBig: From<T>,
    RBig: TryFrom<QO>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    let r_scale =
        RBig::try_from(scale).map_err(|_| err!(MakeMeasurement, "scale must be finite"))?;

    Measurement::new(
        input_domain,
        if scale.is_zero() {
            Function::new(move |x: &Vec<T>| x.clone())
        } else {
            Function::new_fallible(move |x: &Vec<T>| {
                x.iter()
                    .cloned()
                    .map(IBig::from)
                    .map(|x_i| Ok(x_i + sample_discrete_laplace(r_scale.clone())?))
                    // postprocess
                    .map(|res| res.map(T::saturating_cast))
                    .collect()
            })
        },
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(laplace_map(scale, QO::zero())),
    )
}

#[cfg(test)]
mod test;
