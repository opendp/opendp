use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{BoundedRange, MaxDivergence},
    traits::Float,
};

/// Constructs a new output measurement where the output measure
/// is casted from `MaxDivergence<QO>` to `BoundedRange<QO>`.
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
/// * `QO` - Output distance type. One of `f32` or `f64`.
/// For more details, see: https://differentialprivacy.org/exponential-mechanism-bounded-range/

pub fn make_bounded_range_to_pureDP<DI, TO, MI>(
    m: Measurement<DI, TO, MI, BoundedRange>,
) -> Fallible<Measurement<DI, TO, MI, MaxDivergence>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map: PrivacyMap<MI, BoundedRange> = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in).map(|br_eps| br_eps)
        }),
    )
}
