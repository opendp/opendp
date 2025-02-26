use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{MaxDivergence, RangeDivergence},
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(meas(rust_type = "AnyMeasurement")),
    generics(DI(suppress), TO(suppress), MI(suppress))
)]
/// Constructs a new output measurement where the output measure
/// is converted from `BoundedRange` to `MaxDivergence`.
///
/// For more details, see: https://differentialprivacy.org/exponential-mechanism-bounded-range/
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be converted
///
/// # Generics
/// * `DI` - Input Domain
/// * `DO` - Output Domain
/// * `MI` - Input Metric
pub fn make_bounded_range_to_pureDP<DI, TO, MI>(
    meas: Measurement<DI, TO, MI, RangeDivergence>,
) -> Fallible<Measurement<DI, TO, MI, MaxDivergence>>
where
    DI: Domain,
    MI: 'static + Metric,
    (DI, MI): MetricSpace,
{
    let privacy_map: PrivacyMap<MI, RangeDivergence> = meas.privacy_map.clone();
    meas.with_map(
        meas.input_metric.clone(),
        MaxDivergence,
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| privacy_map.eval(d_in)),
    )
}
