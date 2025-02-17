use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measure, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::{ffi::ExtrinsicDivergence, MaxDivergence, ZeroConcentratedDivergence},
};

#[bootstrap(features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is δ-approximate, where δ=0.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
fn make_approximate(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    fn monomorphize<MO: 'static + Measure>(
        measurement: &AnyMeasurement,
    ) -> Fallible<AnyMeasurement> {
        let privacy_map = measurement.privacy_map.clone();
        let measurement = measurement.with_map(
            measurement.input_metric.clone(),
            try_!(measurement.output_measure.clone().downcast::<MO>()),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in)?.downcast::<MO::Distance>()
            }),
        )?;

        let m = super::make_approximate(measurement)?;

        let privacy_map = m.privacy_map.clone();
        m.with_map(
            m.input_metric.clone(),
            AnyMeasure::new(m.output_measure.clone()),
            PrivacyMap::new_fallible(move |d_in: &AnyObject| {
                privacy_map.eval(d_in).map(AnyObject::new)
            }),
        )
    }

    dispatch!(
        monomorphize,
        [(
            measurement.output_measure.type_,
            [
                MaxDivergence,
                ZeroConcentratedDivergence,
                ExtrinsicDivergence
            ]
        )],
        (measurement)
    )
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_approximate(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    FfiResult::from(make_approximate(try_as_ref!(measurement)))
}
