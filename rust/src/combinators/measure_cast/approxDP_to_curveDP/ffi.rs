use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, PrivacyMap},
    error::Fallible,
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::{Approximate, PureDP},
};

fn make_approxDP_to_curveDP(measurement: &AnyMeasurement) -> Fallible<AnyMeasurement> {
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

    let m = super::make_approxDP_to_curveDP(measurement)?;

    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        AnyMeasure::new(m.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in).map(AnyObject::new)
        }),
    )
}

#[bootstrap(name = "make_approxDP_to_curveDP", features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is casted from `ApproxDP` to `PrivacyCurveDP`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_approxDP_to_curveDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    make_approxDP_to_curveDP(try_as_ref!(measurement)).into()
}

#[bootstrap(name = "make_fixed_approxDP_to_approxDP", features("contrib"))]
#[deprecated(since = "0.15.0", note = "Use `make_approxDP_to_curveDP` instead.")]
/// Deprecated alias for `make_approxDP_to_curveDP`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_fixed_approxDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    opendp_combinators__make_approxDP_to_curveDP(measurement)
}
