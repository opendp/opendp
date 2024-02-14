use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::ZeroConcentratedDivergence,
    traits::Float,
};

#[bootstrap(features("contrib"), dependencies("$get_dependencies(measurement)"))]
/// Constructs a new output measurement where the output measure
/// is casted from `ZeroConcentratedDivergence<QO>` to `SmoothedMaxDivergence<QO>`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
fn make_zCDP_to_approxDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    fn monomorphize<QO: Float>(m: &AnyMeasurement) -> Fallible<AnyMeasurement> {
        let privacy_map = m.privacy_map.clone();
        let measurement = Measurement::new(
            m.input_domain.clone(),
            m.function.clone(),
            m.input_metric.clone(),
            try_!(m
                .output_measure
                .clone()
                .downcast::<ZeroConcentratedDivergence<QO>>()),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in)?.downcast::<QO>()
            }),
        )?;

        let m = super::make_zCDP_to_approxDP(measurement)?;

        let privacy_map = m.privacy_map.clone();
        m.with_map(
            m.input_metric.clone(),
            AnyMeasure::new(m.output_measure.clone()),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in).map(AnyObject::new)
            }),
        )
    }

    let Q = measurement.output_measure.distance_type.clone();

    dispatch!(monomorphize, [
        (Q, @floats)
    ], (measurement))
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_zCDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    FfiResult::from(make_zCDP_to_approxDP(try_as_ref!(measurement)))
}
