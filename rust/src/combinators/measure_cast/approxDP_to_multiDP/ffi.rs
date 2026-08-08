use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::{Approximate, PureDP},
};

#[allow(non_snake_case)]
fn make_approxDP_to_multiDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
    let privacy_map = measurement.privacy_map.clone();

    let measurement = measurement.with_map(
        measurement.input_metric.clone(),
        measurement
            .output_measure
            .clone()
            .downcast::<Approximate<PureDP>>()?,
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in)?.downcast::<(f64, f64)>()
        }),
    )?;

    let measurement = super::make_approxDP_to_multiDP(measurement)?;
    let privacy_map = measurement.privacy_map.clone();
    measurement.with_map(
        measurement.input_metric.clone(),
        AnyMeasure::new(measurement.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in).map(AnyObject::new)
        }),
    )
}

#[bootstrap(name = "make_approxDP_to_multiDP", features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is cast from `ApproxDP` to `MultiDP`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be cast
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C" fn opendp_combinators__make_approxDP_to_multiDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    make_approxDP_to_multiDP(try_as_ref!(measurement)).into()
}

#[bootstrap(name = "make_fixed_approxDP_to_approxDP", features("contrib"))]
#[deprecated(since = "0.15.0", note = "Use `make_approxDP_to_multiDP` instead.")]
/// Deprecated compatibility alias for `make_approxDP_to_multiDP`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be cast
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C" fn opendp_combinators__make_fixed_approxDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    opendp_combinators__make_approxDP_to_multiDP(measurement)
}
