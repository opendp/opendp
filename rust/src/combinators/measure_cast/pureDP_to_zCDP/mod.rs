use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    traits::{InfDiv, InfPowI},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Constructs a new output measurement where the output measure
/// is casted from `MaxDivergence` to `ZeroConcentratedDivergence`.
///
/// # Citations
/// - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
///
/// # Arguments
/// * `meas` - a measurement with a privacy curve to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `TO` - Output Type
/// * `MI` - Input Metric
pub fn make_pureDP_to_zCDP<DI, TO, MI>(
    m: Measurement<DI, TO, MI, MaxDivergence>,
) -> Fallible<Measurement<DI, TO, MI, ZeroConcentratedDivergence>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map
                .eval(d_in)
                .and_then(|eps| eps.inf_powi(2.into())?.inf_div(&2.0))
        }),
    )
}
