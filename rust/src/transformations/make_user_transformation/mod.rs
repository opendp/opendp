use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, StabilityMap, Transformation},
    ffi::any::{wrap_func, AnyDomain, AnyMetric, AnyTransformation, CallbackFn},
};

#[bootstrap(
    name = "make_user_transformation",
    features("contrib", "honest-but-curious"),
    arguments(
        input_domain(hint = "Domain"),
        input_metric(hint = "Metric"),
        output_domain(hint = "Domain"),
        output_metric(hint = "Metric"),
        function(rust_type = "$domain_carrier_type(output_domain)"),
        stability_map(rust_type = "$metric_distance_type(output_metric)"),
    ),
    dependencies(
        "input_domain",
        "input_metric",
        "output_domain",
        "output_metric",
        "c_function",
        "c_stability_map"
    )
)]
/// Construct a Transformation from user-defined callbacks.
///
/// # Arguments
/// * `input_domain` - A domain describing the set of valid inputs for the function.
/// * `input_metric` - The metric from which distances between adjacent inputs are measured.
/// * `output_domain` - A domain describing the set of valid outputs of the function.
/// * `output_metric` - The metric from which distances between outputs of adjacent inputs are measured.
/// * `function` - A function mapping data from `input_domain` to `output_domain`.
/// * `stability_map` - A function mapping distances from `input_metric` to `output_metric`.
#[no_mangle]
pub extern "C" fn opendp_transformations__make_user_transformation(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_domain: *const AnyDomain,
    output_metric: *const AnyMetric,
    function: CallbackFn,
    stability_map: CallbackFn,
) -> FfiResult<*mut AnyTransformation> {
    Transformation::new(
        try_as_ref!(input_domain).clone(),
        try_as_ref!(output_domain).clone(),
        Function::new_fallible(wrap_func(function)),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_metric).clone(),
        StabilityMap::new_fallible(wrap_func(stability_map)),
    )
    .into()
}
