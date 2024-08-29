use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::{PrivacyProfile, SmoothedMaxDivergence},
};

#[bootstrap(
    features("contrib"),
    arguments(measurement(rust_type = b"null"),),
    dependencies("$get_dependencies(measurement)")
)]
/// Fix the delta parameter in the privacy map of a `measurement` with a SmoothedMaxDivergence output measure.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy curve to be fixed
/// * `delta` - parameter to fix the privacy curve with
fn make_fix_delta(measurement: &AnyMeasurement, delta: f64) -> Fallible<AnyMeasurement> {
    let privacy_map = measurement.privacy_map.clone();
    let meas = Measurement::new(
        measurement.input_domain.clone(),
        measurement.function.clone(),
        measurement.input_metric.clone(),
        measurement
            .output_measure
            .downcast_ref::<SmoothedMaxDivergence>()?
            .clone(),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in)?.downcast::<PrivacyProfile>()
        }),
    )?;
    let meas = super::make_fix_delta(&meas, delta)?;
    let privacy_map = meas.privacy_map.clone();
    Measurement::new(
        meas.input_domain.clone(),
        meas.function.clone(),
        meas.input_metric.clone(),
        AnyMeasure::new(meas.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            Ok(AnyObject::new(privacy_map.eval(d_in)?))
        }),
    )
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_fix_delta(
    measurement: *const AnyMeasurement,
    delta: f64,
) -> FfiResult<*mut AnyMeasurement> {
    // run combinator on measurement
    make_fix_delta(try_as_ref!(measurement), delta).into()
}
