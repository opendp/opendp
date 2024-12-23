use crate::{
    core::{FfiResult, PrivacyMap},
    ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
    measures::BoundedRange,
};

#[no_mangle]
pub extern "C" fn opendp_combinators__make_bounded_range_to_pureDP(
    measurement: *const AnyMeasurement,
) -> FfiResult<*mut AnyMeasurement> {
    let measurement = try_as_ref!(measurement);
    let privacy_map = measurement.privacy_map.clone();
    let measurement = try_!(measurement.with_map(
        measurement.input_metric.clone(),
        try_!(measurement
            .output_measure
            .clone()
            .downcast::<BoundedRange>()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in)?.downcast::<f64>()),
    ));

    let m = try_!(super::make_bounded_range_to_pureDP(measurement));

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
