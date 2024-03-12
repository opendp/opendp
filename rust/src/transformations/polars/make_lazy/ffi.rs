use crate::{
    core::FfiResult,
    domains::DataFrameDomain,
    ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
};

use super::make_lazy;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_lazy(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<DataFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);

    let (input_domain, output_domain, function, input_metric, output_metric, stability_map) =
        try_!(make_lazy(input_domain, input_metric.clone())).decompose();

    AnyTransformation::new(
        AnyDomain::new(input_domain),
        AnyDomain::new(output_domain),
        function.into_any(),
        input_metric,
        output_metric,
        stability_map,
    )
    .into()
}
