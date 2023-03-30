use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap, StabilityMap, Transformation},
    error::Fallible,
    ffi::{
        any::{
            AnyDomain, AnyFunction, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject,
            AnyTransformation,
        },
        util,
    },
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
/// * `input_domain` - A domain describing the set of valid inputs for the function.
/// * `output_domain` - A domain describing the set of valid outputs of the function.
/// * `function` - A function mapping data from `input_domain` to `output_domain`.
/// * `input_metric` - The metric from which distances between adjacent inputs are measured.
/// * `output_metric` - The metric from which distances between outputs of adjacent inputs are measured.
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
        function(rust_type = "$pass_through(TO)"),
        input_metric(hint = "Metric"),
        output_measure(hint = "Measure"),
        privacy_map(rust_type = "$measure_distance_type(output_measure)"),
    ),
    dependencies("c_function", "c_privacy_map")
)]
/// Construct a Measurement from user-defined callbacks.
///
/// # Arguments
/// * `input_domain` - A domain describing the set of valid inputs for the function.
/// * `function` - A function mapping data from `input_domain` to a release of type `TO`.
/// * `input_metric` - The metric from which distances between adjacent inputs are measured.
/// * `output_measure` - The measure from which distances between adjacent output distributions are measured.
/// * `privacy_map` - A function mapping distances from `input_metric` to `output_measure`.
///
/// # Generics
/// * `TO` - The data type of outputs from the function.
#[allow(dead_code)]
fn make_user_measurement<TO>(
    input_domain: AnyDomain,
    function: CallbackFn,
    input_metric: AnyMetric,
    output_measure: AnyMeasure,
    privacy_map: CallbackFn,
) -> Fallible<AnyMeasurement> {
    let _ = (
        input_domain,
        function,
        input_metric,
        output_measure,
        privacy_map,
    );
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_user_measurement(
    input_domain: *const AnyDomain,
    function: CallbackFn,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    privacy_map: CallbackFn,
    TO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let _TO = TO;
    FfiResult::Ok(util::into_raw(Measurement::new(
        try_as_ref!(input_domain).clone(),
        Function::new_fallible(wrap_func(function)),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_measure).clone(),
        PrivacyMap::new_fallible(wrap_func(privacy_map)),
    )))
}

#[bootstrap(
    name = "make_user_postprocessor",
    features("contrib"),
    arguments(function(rust_type = "$pass_through(TO)")),
    dependencies("c_function")
)]
/// Construct a Postprocessor from user-defined callbacks.
///
/// # Arguments
/// * `function` - A function mapping data to a value of type `TO`
///
/// # Generics
/// * `TO` - Output Type
#[allow(dead_code)]
fn make_user_postprocessor<TO>(function: CallbackFn) -> Fallible<AnyFunction> {
    let _ = function;
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_user_postprocessor(
    function: CallbackFn,
    TO: *const c_char,
) -> FfiResult<*mut AnyFunction> {
    let _TO = TO;
    FfiResult::Ok(util::into_raw(Function::new_fallible(wrap_func(function))))
}
