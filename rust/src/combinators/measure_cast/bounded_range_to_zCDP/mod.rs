use dashu::ibig;
use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{RangeDivergence, ZeroConcentratedDivergence},
    traits::{InfDiv, InfPowI},
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(meas(rust_type = "AnyMeasurement")),
    generics(DI(suppress), TO(suppress), MI(suppress))
)]
/// Constructs a new output measurement where the output measure
/// is converted from `RangeDivergence` to `ZeroConcentratedDivergence`.
///
/// For more details, see: https://differentialprivacy.org/exponential-mechanism-bounded-range/
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be converted
///
/// # Generics
/// * `DI` - Input Domain
/// * `MI` - Input Metric
/// * `TO` - Output Type
pub fn make_bounded_range_to_zCDP<DI, MI, TO>(
    meas: Measurement<DI, TO, MI, RangeDivergence>,
) -> Fallible<Measurement<DI, TO, MI, ZeroConcentratedDivergence>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map: PrivacyMap<MI, RangeDivergence> = meas.privacy_map.clone();

    meas.with_map(
        meas.input_metric.clone(),
        ZeroConcentratedDivergence,
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            privacy_map.eval(d_in)?.inf_powi(ibig!(2))?.inf_div(&8.0)
        }),
    )
}
