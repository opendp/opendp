use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, PrivacyProfile, ProfileDP, PureDP},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// Constructs a new output measurement where the output measure
/// is casted from `Approximate<PureDP>` to `ProfileDP`
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
pub fn make_approxDP_to_profileDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, Approximate<PureDP>, TO>,
) -> Fallible<Measurement<DI, MI, ProfileDP, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = measurement.privacy_map.clone();
    measurement.with_map(
        measurement.input_metric.clone(),
        ProfileDP::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in).and_then(|(eps, delta)| {
                PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(eps, delta)])
            })
        }),
    )
}

#[deprecated(since = "0.15.0", note = "Use `make_approxDP_to_profileDP` instead.")]
/// Deprecated compatibility alias for [`make_approxDP_to_profileDP`].
#[allow(non_snake_case)]
pub fn make_fixed_approxDP_to_approxDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, Approximate<PureDP>, TO>,
) -> Fallible<Measurement<DI, MI, ProfileDP, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    make_approxDP_to_profileDP(measurement)
}
