use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap},
    error::Fallible,
    ffi::any::{
        wrap_func, AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, CallbackFn,
    },
};

#[bootstrap(
    name = "make_user_measurement",
    features("contrib", "honest-but-curious"),
    arguments(
        input_domain(hint = "Domain"),
        input_metric(hint = "Metric"),
        output_measure(hint = "Measure"),
        function(rust_type = "$pass_through(TO)"),
        privacy_map(rust_type = "$measure_distance_type(output_measure)"),
    ),
    generics(TO(default = "ExtrinsicObject")),
    dependencies(
        "input_domain",
        "input_metric",
        "output_measure",
        "c_function",
        "c_privacy_map"
    )
)]
/// Construct a Measurement from user-defined callbacks.
///
/// # Arguments
/// * `input_domain` - A domain describing the set of valid inputs for the function.
/// * `input_metric` - The metric from which distances between adjacent inputs are measured.
/// * `output_measure` - The measure from which distances between adjacent output distributions are measured.
/// * `function` - A function mapping data from `input_domain` to a release of type `TO`.
/// * `privacy_map` - A function mapping distances from `input_metric` to `output_measure`.
///
/// # Generics
/// * `TO` - The data type of outputs from the function.
#[allow(dead_code)]
fn make_user_measurement<TO>(
    input_domain: AnyDomain,
    input_metric: AnyMetric,
    output_measure: AnyMeasure,
    function: CallbackFn,
    privacy_map: CallbackFn,
) -> Fallible<Measurement<AnyDomain, AnyObject, AnyMetric, AnyMeasure>> {
    let _ = (
        input_domain,
        input_metric,
        output_measure,
        privacy_map,
        function,
    );
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_measurements__make_user_measurement(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    function: CallbackFn,
    privacy_map: CallbackFn,
    TO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let _TO = TO;
    Measurement::new(
        try_as_ref!(input_domain).clone(),
        Function::new_fallible(wrap_func(function)),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_measure).clone(),
        PrivacyMap::new_fallible(wrap_func(privacy_map)),
    )
    .into()
}
