use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MultiDP, PrivacyGuarantee, PureDP},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// Constructs a new output measurement where the output measure
/// is cast from `Approximate<PureDP>` to `MultiDP`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be cast
///
/// # Generics
/// * `DI` - Input Domain
/// * `MI` - Input Metric
/// * `TO` - Output Type
#[allow(non_snake_case)]
pub fn make_approxDP_to_multiDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, Approximate<PureDP>, TO>,
) -> Fallible<Measurement<DI, MI, MultiDP, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = measurement.privacy_map.clone();
    measurement.with_map(
        measurement.input_metric.clone(),
        MultiDP,
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let (fixed_epsilon, fixed_delta) = privacy_map.eval(d_in)?;
            PrivacyGuarantee::new().with_approxDP(vec![(fixed_epsilon, fixed_delta)])
        }),
    )
}

#[deprecated(since = "0.15.0", note = "Use `make_approxDP_to_multiDP`.")]
/// Deprecated compatibility alias for [`make_approxDP_to_multiDP`].
#[allow(non_snake_case)]
pub fn make_fixed_approxDP_to_approxDP<DI, MI, TO>(
    measurement: Measurement<DI, MI, Approximate<PureDP>, TO>,
) -> Fallible<Measurement<DI, MI, MultiDP, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    make_approxDP_to_multiDP(measurement)
}
