use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MaxDivergence, PrivacyProfile, SmoothedMaxDivergence},
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
pub fn make_fixed_approxDP_to_approxDP<DI, TO, MI>(
    measurement: Measurement<DI, TO, MI, Approximate<MaxDivergence>>,
) -> Fallible<Measurement<DI, TO, MI, SmoothedMaxDivergence>>
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
                .map(|(eps, delta)| PrivacyProfile::new(fixed_approx_dp_privacy_curve(eps, delta)))
        }),
    )
}

fn fixed_approx_dp_privacy_curve(
    fixed_epsilon: f64,
    fixed_delta: f64,
) -> impl Fn(f64) -> Fallible<f64> {
    move |epsilon: f64| {
        Ok(if epsilon >= fixed_epsilon {
            fixed_delta
        } else {
            1.0
        })
    }
}
