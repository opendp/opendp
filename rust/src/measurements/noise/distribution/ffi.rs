use std::ffi::{CString, c_char};

use crate::core::FfiResult;
use crate::ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::measurements::noise::distribution::{
    gaussian::ffi::opendp_measurements__make_gaussian,
    laplace::ffi::opendp_measurements__make_laplace,
};
use crate::measures::{AccountingPreference, MaxDivergence, MultiDP, ZeroConcentratedDivergence};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_noise(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    scale: f64,
    k: *const i32,
) -> FfiResult<*mut AnyMeasurement> {
    let measure = try_as_ref!(output_measure);
    let output_type = &measure.type_;
    let metric_type = &try_as_ref!(input_metric).type_;
    let preference = if output_type.id == Type::of::<MaxDivergence>().id {
        AccountingPreference::PureDp
    } else if output_type.id == Type::of::<ZeroConcentratedDivergence>().id {
        AccountingPreference::Zcdp
    } else if output_type.id == Type::of::<MultiDP>().id {
        match &metric_type.contents {
            TypeContents::GENERIC {
                name: "L1Distance", ..
            } => AccountingPreference::PureDp,
            TypeContents::GENERIC {
                name: "L2Distance", ..
            } => AccountingPreference::Zcdp,
            TypeContents::GENERIC {
                name: "AbsoluteDistance",
                ..
            } => try_!(measure.downcast_ref::<MultiDP>()).preference(),
            _ => return err!(FFI, "unsupported input metric for MultiDP noise").into(),
        }
    } else {
        return err!(FFI, "unsupported output measure for noise").into();
    };

    let constructor: extern "C" fn(
        *const AnyDomain,
        *const AnyMetric,
        f64,
        *const i32,
        *const c_char,
    ) -> FfiResult<*mut AnyMeasurement> = match preference {
        AccountingPreference::PureDp => opendp_measurements__make_laplace,
        AccountingPreference::Zcdp => opendp_measurements__make_gaussian,
        AccountingPreference::Auto => {
            return err!(
                MakeMeasurement,
                "accounting preference is required for MultiDP scalar noise"
            )
            .into();
        }
    };
    let descriptor =
        try_!(CString::new(output_type.descriptor.as_str()).map_err(|e| err!(FFI, "{e}")));
    let result = constructor(input_domain, input_metric, scale, k, descriptor.as_ptr());
    let mut measurement = try_!(crate::error::Fallible::from(result));
    measurement.output_measure = measure.clone();
    Ok(measurement).into()
}
