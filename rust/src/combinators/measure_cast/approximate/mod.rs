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
/// * `DO` - Output Domain
/// * `MI` - Input Metric
/// * `MO` - Output Measure
pub fn make_approximate<DI, TO, MI, MO>(
    m: Measurement<DI, TO, MI, MO>,
) -> Fallible<Measurement<DI, TO, MI, Approximate<MO>>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
    (DI, MI): MetricSpace,
{
    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        Approximate::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in).map(|d_out| (d_out, 0.0))
        }),
    )
}
