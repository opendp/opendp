use std::ffi::{CString, c_char, c_void};

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric},
        util::{Type, TypeContents},
    },
    measurements::noise_threshold::distribution::{
        gaussian::ffi::opendp_measurements__make_gaussian_threshold,
        laplace::ffi::opendp_measurements__make_laplace_threshold,
    },
    measures::{Approximate, MaxDivergence, MultiDP, ZeroConcentratedDivergence},
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
    let measure = try_as_ref!(output_measure);
    let output_type = &measure.type_;
    let metric_type = &try_as_ref!(input_metric).type_;
    let constructor: extern "C" fn(
        *const AnyDomain,
        *const AnyMetric,
        f64,
        *const c_void,
        *const i32,
        *const c_char,
    ) -> FfiResult<*mut AnyMeasurement> = if output_type.id
        == Type::of::<Approximate<MaxDivergence>>().id
    {
        opendp_measurements__make_laplace_threshold
    } else if output_type.id == Type::of::<Approximate<ZeroConcentratedDivergence>>().id {
        opendp_measurements__make_gaussian_threshold
    } else if output_type.id == Type::of::<MultiDP>().id {
        match &metric_type.contents {
            TypeContents::GENERIC {
                name: "L01InfDistance",
                ..
            } => opendp_measurements__make_laplace_threshold,
            TypeContents::GENERIC {
                name: "L02InfDistance",
                ..
            } => opendp_measurements__make_gaussian_threshold,
            _ => return err!(FFI, "unsupported input metric for MultiDP threshold noise").into(),
        }
    } else {
        return err!(FFI, "unsupported output measure for threshold noise").into();
    };
    let descriptor =
        try_!(CString::new(output_type.descriptor.as_str()).map_err(|e| err!(FFI, "{e}")));
    let result = constructor(
        input_domain,
        input_metric,
        scale,
        threshold,
        k,
        descriptor.as_ptr(),
    );
    let mut measurement = try_!(crate::error::Fallible::from(result));
    measurement.output_measure = measure.clone();
    Ok(measurement).into()
}
