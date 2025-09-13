use crate::{
    core::FfiResult,
    ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyOdometer},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_privacy_filter(
    odometer: *const AnyOdometer,
    d_in: *const AnyObject,
    d_out: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let odometer = try_as_ref!(odometer).clone();
    let d_in = try_as_ref!(d_in).clone();
    let d_out = try_as_ref!(d_out).clone();
    super::make_privacy_filter::<AnyDomain, AnyMetric, AnyMeasure, AnyObject, AnyObject>(
        odometer, d_in, d_out,
    )
    .map(|m| m.into_any_out())
    .into()
}
