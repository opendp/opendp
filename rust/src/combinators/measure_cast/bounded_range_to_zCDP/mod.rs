use dashu::ibig;

use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{BoundedRange, ZeroConcentratedDivergence},
    traits::{InfDiv, InfPowI},
};

/// Constructs a new output measurement where the output measure
/// is casted from `BoundedRange` to `ZeroConcentratedDivergence`.
///
/// For more details, see: https://differentialprivacy.org/exponential-mechanism-bounded-range/
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be casted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
pub fn make_bounded_range_to_zCDP<DI, TO, MI>(
    meas: Measurement<DI, TO, MI, BoundedRange>,
) -> Fallible<Measurement<DI, TO, MI, ZeroConcentratedDivergence>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map: PrivacyMap<MI, BoundedRange> = meas.privacy_map.clone();

    meas.with_map(
        meas.input_metric.clone(),
        ZeroConcentratedDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in)?.inf_powi(ibig!(2))?.inf_div(&8.0)
        }),
    )
}
