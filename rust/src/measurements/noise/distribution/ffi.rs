use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric},
        util::{Type, into_c_char_p},
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
    if *MO == Type::of::<ZeroConcentratedDivergence>() {
        opendp_measurements__make_gaussian(
            input_domain,
            input_metric,
            scale,
            k,
            try_!(into_c_char_p(MO.descriptor.clone())),
        )
    } else if *MO == Type::of::<MaxDivergence>() {
        opendp_measurements__make_laplace(
            input_domain,
            input_metric,
            scale,
            k,
            try_!(into_c_char_p(MO.descriptor.clone())),
        )
    } else {
        err!(
            FFI,
            "output_measure must be MaxDivergence or ZeroConcentratedDivergence"
        )
        .into()
    }
}
