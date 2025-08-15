use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::Approximate,
};

#[cfg(feature = "ffi")]
mod ffi;

/// Constructs a new output measurement where the output measure
/// is δ-approximate, where δ=0.
///
/// # Arguments
/// * `m` - a measurement
///
/// # Generics
/// * `DI` - Input Domain
/// * `MI` - Input Metric
/// * `MO` - Output Measure
/// * `TO` - Output Type
pub fn make_approximate<DI, MI, MO, TO>(
    m: Measurement<DI, MI, MO, TO>,
) -> Fallible<Measurement<DI, MI, Approximate<MO>, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        Approximate(m.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in).map(|d_out| (d_out, 0.0))
        }),
    )
}
