use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap, StabilityMap, Transformation},
    error::Fallible,
    ffi::{
        any::{
            AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyTransformation,
            IntoAnyStabilityMapExt,
        },
        util,
    },
    metrics::AgnosticMetric,
};

type CallbackFn = extern "C" fn(*const AnyObject) -> *mut FfiResult<*mut AnyObject>;

// wrap a CallbackFn in a closure, so that it can be used in transformations and measurements
fn wrap_func(func: CallbackFn) -> impl Fn(&AnyObject) -> Fallible<AnyObject> {
    move |arg: &AnyObject| -> Fallible<AnyObject> {
        util::into_owned(func(arg as *const AnyObject))?.into()
    }
}

#[bootstrap(
    name = "make_user_transformation",
    features("contrib", "honest-but-curious"),
    arguments(
        input_domain(hint = "Domain"),
        output_domain(hint = "Domain"),
        function(rust_type = "$domain_carrier_type(output_domain)"),
        input_metric(hint = "Metric"),
        output_metric(hint = "Metric"),
        stability_map(rust_type = "$metric_distance_type(output_metric)"),
    ),
    dependencies("c_function", "c_stability_map")
)]
/// Construct a Transformation from user-defined callbacks.
///
/// # Arguments
/// * `function` - A function mapping data from `input_domain` to `output_domain`.
/// * `stability_map` - A function mapping distances from `input_metric` to `output_metric`.
#[no_mangle]
pub extern "C" fn opendp_combinators__make_user_transformation(
    input_domain: *const AnyDomain,
    output_domain: *const AnyDomain,
    function: CallbackFn,
    input_metric: *const AnyMetric,
    output_metric: *const AnyMetric,
    stability_map: CallbackFn,
) -> FfiResult<*mut AnyTransformation> {
    FfiResult::Ok(util::into_raw(Transformation::new(
        try_as_ref!(input_domain).clone(),
        try_as_ref!(output_domain).clone(),
        Function::new_fallible(wrap_func(function)),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_metric).clone(),
        StabilityMap::new_fallible(wrap_func(stability_map)),
    )))
}

#[bootstrap(
    name = "make_user_measurement",
    features("contrib", "honest-but-curious"),
    arguments(
        input_domain(hint = "Domain"),
        output_domain(hint = "Domain"),
        function(rust_type = "$domain_carrier_type(output_domain)"),
        input_metric(hint = "Metric"),
        output_measure(hint = "Measure"),
        privacy_map(rust_type = "$measure_distance_type(output_measure)"),
    ),
    dependencies("c_function", "c_privacy_map")
)]
/// Construct a Measurement from user-defined callbacks.
///
/// # Arguments
/// * `function` - A function mapping data from `input_domain` to `output_domain`.
/// * `privacy_map` - A function mapping distances from `input_metric` to `output_measure`.
#[no_mangle]
pub extern "C" fn opendp_combinators__make_user_measurement(
    input_domain: *const AnyDomain,
    output_domain: *const AnyDomain,
    function: CallbackFn,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    privacy_map: CallbackFn,
) -> FfiResult<*mut AnyMeasurement> {
    FfiResult::Ok(util::into_raw(Measurement::new(
        try_as_ref!(input_domain).clone(),
        try_as_ref!(output_domain).clone(),
        Function::new_fallible(wrap_func(function)),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_measure).clone(),
        PrivacyMap::new_fallible(wrap_func(privacy_map)),
    )))
}

#[bootstrap(
    name = "make_user_postprocessor",
    features("contrib"),
    arguments(
        input_domain(hint = "Domain"),
        output_domain(hint = "Domain"),
        function(rust_type = "$domain_carrier_type(output_domain)"),
    ),
    dependencies("c_function")
)]
/// Construct a Postprocessor from user-defined callbacks.
///
/// # Arguments
/// * `function` - A function mapping data from `input_domain` to `output_domain`.
#[no_mangle]
pub extern "C" fn opendp_combinators__make_user_postprocessor(
    input_domain: *const AnyDomain,
    output_domain: *const AnyDomain,
    function: CallbackFn,
) -> FfiResult<*mut AnyTransformation> {
    FfiResult::Ok(util::into_raw(Transformation::new(
        try_as_ref!(input_domain).clone(),
        try_as_ref!(output_domain).clone(),
        Function::new_fallible(wrap_func(function)),
        AnyMetric::new(AgnosticMetric::default()),
        AnyMetric::new(AgnosticMetric::default()),
        StabilityMap::<AgnosticMetric, AgnosticMetric>::new(|_| ()).into_any(),
    )))
}
