use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::MaxDivergence,
};

#[bootstrap(features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is casted from `MaxDivergence` to `ZeroConcentratedDivergence`.
///
/// # Citations
/// - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
fn make_pureDP_to_zCDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    let privacy_map = measurement.privacy_map.clone();
    let measurement = measurement.with_map(
        measurement.input_metric.clone(),
        measurement
            .output_measure
            .clone()
            .downcast::<MaxDivergence>()?,
        PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in)?.downcast::<f64>()),
    )?;

    let m = super::make_pureDP_to_zCDP(measurement)?;

    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        AnyMeasure::new(m.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in).map(AnyObject::new)
        }),
    )
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_pureDP_to_zCDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    FfiResult::from(make_pureDP_to_zCDP(try_as_ref!(measurement)))
}
