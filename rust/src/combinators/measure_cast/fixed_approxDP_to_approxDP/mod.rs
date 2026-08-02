use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MaxDivergence, PrivacyGuarantee, SmoothedMaxDivergence},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// Constructs a new output measurement where the output measure
/// is casted from `Approximate<MaxDivergence>` to `SmoothedMaxDivergence`
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
pub fn make_fixed_approxDP_to_approxDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, Approximate<MaxDivergence>, TO>,
) -> Fallible<Measurement<DI, MI, SmoothedMaxDivergence, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = measurement.privacy_map.clone();
    measurement.with_map(
        measurement.input_metric.clone(),
        SmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map
                .eval(d_in)
                .and_then(|(eps, delta)| PrivacyGuarantee::new().with_approxDP(vec![(eps, delta)]))
        }),
    )
}
