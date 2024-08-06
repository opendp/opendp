use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, PrivacyMap},
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::{Approximate, MaxDivergence},
};

#[bootstrap(name = "make_fixed_approxDP_to_approxDP", features("contrib"))]
/// Constructs a new output measurement where the output measure
/// is casted from `Approximate<MaxDivergence>` to `SmoothedMaxDivergence`.
///
/// # Arguments
/// * `measurement` - a measurement with a privacy measure to be casted
#[no_mangle]
pub extern "C" fn opendp_combinators__make_fixed_approxDP_to_approxDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    let measurement = try_as_ref!(measurement);
    let privacy_map = measurement.privacy_map.clone();

    let measurement = try_!(measurement.with_map(
        measurement.input_metric.clone(),
        try_!(measurement
            .output_measure
            .clone()
            .downcast::<Approximate<MaxDivergence>>()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map
            .eval(d_in)?
            .downcast::<(f64, f64)>()),
    ));

    let m = try_!(super::make_fixed_approxDP_to_approxDP(measurement));

    let privacy_map = m.privacy_map.clone();
    m.with_map(
        m.input_metric.clone(),
        AnyMeasure::new(m.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in).map(AnyObject::new)
        }),
    )
    .into()
}
