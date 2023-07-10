use std::ffi::c_char;

use crate::{
    core::FfiResult,
    domains::DataFrameDomain,
    ffi::{any::{AnyDomain, AnyMetric, AnyTransformation, Downcast, IntoAnyFunctionExt}, util::to_str},
};

use super::make_column;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_column(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    column_name: *mut c_char
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<DataFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let column_name = try_!(to_str(column_name)).to_string();

    let (input_domain, output_domain, function, input_metric, output_metric, stability_map) =
        try_!(make_column(input_domain, input_metric.clone(), column_name)).decompose();

    AnyTransformation::new(
        AnyDomain::new(input_domain),
        AnyDomain::new(output_domain),
        function.into_any(),
        input_metric,
        output_metric,
        stability_map
    ).into()
}
