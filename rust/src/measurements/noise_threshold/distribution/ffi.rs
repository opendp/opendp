use std::ffi::c_void;

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric},
        util::{Type, into_c_char_p},
    },
    measurements::noise_threshold::distribution::{
        gaussian::ffi::opendp_measurements__make_gaussian_threshold,
        laplace::ffi::opendp_measurements__make_laplace_threshold,
    },
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_noise_threshold(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    scale: f64,
    threshold: *const c_void,
    k: *const i32,
) -> FfiResult<*mut AnyMeasurement> {
    let MO = &try_as_ref!(output_measure).type_;
    if *MO == Type::of::<Approximate<ZeroConcentratedDivergence>>() {
        opendp_measurements__make_gaussian_threshold(
            input_domain,
            input_metric,
            scale,
            threshold,
            k,
            try_!(into_c_char_p(MO.descriptor.clone())),
        )
    } else if *MO == Type::of::<Approximate<MaxDivergence>>() {
        opendp_measurements__make_laplace_threshold(
            input_domain,
            input_metric,
            scale,
            threshold,
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
