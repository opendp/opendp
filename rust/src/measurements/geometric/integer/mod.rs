use dashu::{base::ConversionError, rational::RBig};

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::laplace_map,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{samplers::sample_discrete_laplace_linear, ExactIntCast, Float, InfCast, Integer},
};

/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
/// using a linear-time algorithm on finite data types.
///
/// This algorithm can be executed in constant time if bounds are passed.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
/// * `bounds` - Set bounds on the count to make the algorithm run in constant-time.
///
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `QO` - Data type of the scale and output distance.
pub(crate) fn make_scalar_geometric<T, QO>(
    input_domain: AtomDomain<T>,
    input_metric: AbsoluteDistance<T>,
    scale: QO,
    bounds: Option<(T, T)>,
) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<T>, MaxDivergence<QO>>>
where
    T: Integer,
    QO: Float + InfCast<T>,
    RBig: TryFrom<QO, Error = ConversionError>,
    usize: ExactIntCast<QO::Bits> + ExactIntCast<T>,
    QO::Bits: ExactIntCast<usize>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    if bounds
        .as_ref()
        .map(|(lower, upper)| lower > upper)
        .unwrap_or(false)
    {
        return fallible!(MakeMeasurement, "lower may not be greater than upper");
    }

    Measurement::new(
        input_domain,
        Function::new_fallible(move |v: &T| sample_discrete_laplace_linear(*v, scale, bounds)),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(laplace_map(scale, QO::zero())),
    )
}

/// Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
/// using a linear-time algorithm on finite data types.
///
/// This algorithm can be executed in constant time if bounds are passed.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
/// * `bounds` - Set bounds on the count to make the algorithm run in constant-time.
///
/// # Generics
/// * `D` - Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `QO` - Data type of the scale and output distance.
pub(crate) fn make_vector_geometric<T, QO>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: L1Distance<T>,
    scale: QO,
    bounds: Option<(T, T)>,
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, L1Distance<T>, MaxDivergence<QO>>>
where
    T: Integer,
    QO: Float + InfCast<T>,
    RBig: TryFrom<QO, Error = ConversionError>,
    usize: ExactIntCast<QO::Bits> + ExactIntCast<T>,
    QO::Bits: ExactIntCast<usize>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }
    if bounds
        .as_ref()
        .map(|(lower, upper)| lower > upper)
        .unwrap_or(false)
    {
        return fallible!(MakeMeasurement, "lower may not be greater than upper");
    }

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<T>| {
            arg.iter()
                .map(|v| sample_discrete_laplace_linear(*v, scale, bounds))
                .collect()
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(laplace_map(scale, QO::zero())),
    )
}

#[cfg(test)]
mod test;
