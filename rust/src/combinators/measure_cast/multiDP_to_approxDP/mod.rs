use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MultiDP, PureDP},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Cast a `MultiDP` measurement to fixed approximate pure DP at `delta`.
///
/// This adapter consumes the aggregate privacy guarantee returned by the
/// measurement's map and exposes its epsilon bound at the requested delta.
/// It is used when a context with an approximate-DP accountant receives a
/// measurement whose native output measure is `MultiDP`.
pub fn make_multiDP_to_approxDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, MultiDP, TO>,
    delta: f64,
) -> Fallible<Measurement<DI, MI, Approximate<PureDP>, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = measurement.privacy_map.clone();
    measurement.with_map(
        measurement.input_metric.clone(),
        Approximate(PureDP),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let guarantee = privacy_map.eval(d_in)?;
            // Privacy curves are conservative and may have a discontinuity at
            // the requested delta. Evaluate just above the boundary so the
            // inversion does not reject an exact floating-point endpoint.
            let epsilon = guarantee.epsilon(delta.next_up()).or_else(|_| {
                // At a discontinuous endpoint, the right-limit is the first
                // stable conservative value above the requested delta.
                guarantee.epsilon(delta + 1e-12)
            })?;
            Ok((epsilon, delta))
        }),
    )
}
