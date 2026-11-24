use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::MultiDP,
};

fn make_multiDP_to_approxDP(measurement: &AnyMeasurement, delta: f64) -> Fallible<AnyMeasurement> {
    let privacy_map = measurement.privacy_map.clone();
    let measurement = measurement.with_map(
        measurement.input_metric.clone(),
        measurement
            .output_measure
            .downcast_ref::<MultiDP>()?
            .clone(),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map
                .eval(d_in)?
                .downcast_ref::<crate::measures::PrivacyGuarantee>()
                .map(Clone::clone)
        }),
    )?;
    let measurement = super::make_multiDP_to_approxDP(measurement, delta)?;
    let privacy_map = measurement.privacy_map.clone();
    measurement.with_map(
        measurement.input_metric.clone(),
        AnyMeasure::new(measurement.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in).map(AnyObject::new)
        }),
    )
}

#[bootstrap(name = "make_multiDP_to_approxDP", features("contrib"))]
/// Cast a `MultiDP` measurement to an approximate-DP measurement at a fixed delta.
///
/// # Arguments
/// * `measurement` - Measurement whose privacy map produces a `PrivacyGuarantee`.
/// * `delta` - Delta at which to evaluate the privacy guarantee.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_multiDP_to_approxDP(
    measurement: *const AnyMeasurement,
    delta: f64,
) -> FfiResult<*mut AnyMeasurement> {
    make_multiDP_to_approxDP(try_as_ref!(measurement), delta).into()
}
