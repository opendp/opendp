use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, PrivacyCurve, PrivacyCurveDP, PureDP},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// Constructs a new output measurement where the output measure
/// is casted from `Approximate<PureDP>` to `PrivacyCurveDP`
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
pub fn make_approxDP_to_curveDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, Approximate<PureDP>, TO>,
) -> Fallible<Measurement<DI, MI, PrivacyCurveDP, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = measurement.privacy_map.clone();
    measurement.with_map(
        measurement.input_metric.clone(),
        PrivacyCurveDP,
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in).map(|(fixed_epsilon, fixed_delta)| {
                PrivacyCurve::new_profile(move |epsilon: f64| {
                    Ok(if epsilon >= fixed_epsilon {
                        fixed_delta
                    } else {
                        1.0
                    })
                })
            })
        }),
    )
}

#[deprecated(since = "0.15.0", note = "Use `make_approxDP_to_curveDP`.")]
/// Deprecated alias for `make_approxDP_to_curveDP`.
pub fn make_fixed_approxDP_to_approxDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, Approximate<PureDP>, TO>,
) -> Fallible<Measurement<DI, MI, PrivacyCurveDP, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    make_approxDP_to_curveDP(measurement)
}
