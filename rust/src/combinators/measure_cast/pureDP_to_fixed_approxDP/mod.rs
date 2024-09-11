use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{FixedSmoothedMaxDivergence, MaxDivergence},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Constructs a new output measurement where the output measure
/// is casted from `MaxDivergence` to `FixedSmoothedMaxDivergence`.
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
pub fn make_pureDP_to_fixed_approxDP<DI, TO, MI>(
    m: Measurement<DI, TO, MI, MaxDivergence>,
) -> Fallible<Measurement<DI, TO, MI, FixedSmoothedMaxDivergence>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        FixedSmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in).map(|eps| (eps, 0.0))
        }),
    )
}
