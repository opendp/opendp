use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric},
        util::Type,
    },
    measurements::noise::distribution::{
        gaussian::ffi::opendp_measurements__make_gaussian,
        laplace::ffi::opendp_measurements__make_laplace,
    },
    measures::{MaxDivergence, ZeroConcentratedDivergence},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_noise(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    scale: f64,
    k: *const i32,
) -> FfiResult<*mut AnyMeasurement> {
    let MO = &try_as_ref!(output_measure).type_;
    // The delegated constructor only borrows its type string. Static C strings
    // avoid allocating an owned CString that no caller would reclaim.
    if *MO == Type::of::<ZeroConcentratedDivergence>() {
        opendp_measurements__make_gaussian(
            input_domain,
            input_metric,
            scale,
            k,
            c"ZeroConcentratedDivergence".as_ptr(),
        )
    } else if *MO == Type::of::<MaxDivergence>() {
        opendp_measurements__make_laplace(
            input_domain,
            input_metric,
            scale,
            k,
            c"MaxDivergence".as_ptr(),
        )
    } else {
        err!(
            FFI,
            "output_measure must be MaxDivergence or ZeroConcentratedDivergence"
        )
        .into()
    }
}
