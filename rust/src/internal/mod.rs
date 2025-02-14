/// This code is used to generate a hidden `_internal` module in Python
/// containing APIs that allow for the construction of library primitives like Transformations and Measurements
/// that do not require the "honest-but-curious" flag to be set.
use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap, StabilityMap, Transformation},
    domains::ffi::{ExtrinsicDomain, ExtrinsicElement},
    error::Fallible,
    ffi::{
        any::{
            wrap_func, AnyDomain, AnyFunction, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject,
            AnyTransformation, CallbackFn, Downcast,
        },
        util::{self, ExtrinsicObject},
    },
    measures::ffi::ExtrinsicDivergence,
    metrics::ffi::ExtrinsicDistance,
};

use self::util::to_str;

#[bootstrap(
    name = "_make_measurement",
    arguments(
        input_domain(hint = "Domain"),
        input_metric(hint = "Metric"),
        output_measure(hint = "Measure"),
        function(rust_type = "$pass_through(TO)"),
        privacy_map(rust_type = "$measure_distance_type(output_measure)"),
    ),
    generics(TO(default = "ExtrinsicObject"))
)]
/// Construct a Measurement from user-defined callbacks.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `make_user_measurement`.
///
/// See `make_user_measurement` for correct usage and proof definition for this function.
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
fn _make_measurement<TO>(
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
pub extern "C" fn opendp_internal___make_measurement(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    function: *const CallbackFn,
    privacy_map: *const CallbackFn,
    TO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let _TO = TO;
    Measurement::new(
        try_as_ref!(input_domain).clone(),
        Function::new_fallible(wrap_func(try_as_ref!(function).clone())),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_measure).clone(),
        PrivacyMap::new_fallible(wrap_func(try_as_ref!(privacy_map).clone())),
    )
    .into()
}

#[bootstrap(
    name = "_make_transformation",
    arguments(
        input_domain(hint = "Domain"),
        input_metric(hint = "Metric"),
        output_domain(hint = "Domain"),
        output_metric(hint = "Metric"),
        function(rust_type = "$domain_carrier_type(output_domain)"),
        stability_map(rust_type = "$metric_distance_type(output_metric)"),
    )
)]
/// Construct a Transformation from user-defined callbacks.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `make_user_transformation`.
///
/// See `make_user_transformation` for correct usage and proof definition for this function.
///
/// # Arguments
/// * `input_domain` - A domain describing the set of valid inputs for the function.
/// * `input_metric` - The metric from which distances between adjacent inputs are measured.
/// * `output_domain` - A domain describing the set of valid outputs of the function.
/// * `output_metric` - The metric from which distances between outputs of adjacent inputs are measured.
/// * `function` - A function mapping data from `input_domain` to `output_domain`.
/// * `stability_map` - A function mapping distances from `input_metric` to `output_metric`.
#[no_mangle]
pub extern "C" fn opendp_internal___make_transformation(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_domain: *const AnyDomain,
    output_metric: *const AnyMetric,
    function: *const CallbackFn,
    stability_map: *const CallbackFn,
) -> FfiResult<*mut AnyTransformation> {
    Transformation::new(
        try_as_ref!(input_domain).clone(),
        try_as_ref!(output_domain).clone(),
        Function::new_fallible(wrap_func(try_as_ref!(function).clone())),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_metric).clone(),
        StabilityMap::new_fallible(wrap_func(try_as_ref!(stability_map).clone())),
    )
    .into()
}

#[bootstrap(
    name = "_extrinsic_domain",
    arguments(
        identifier(c_type = "char *", rust_type = b"null"),
        member(rust_type = "bool"),
        descriptor(default = b"null", rust_type = "ExtrinsicObject")
    )
)]
/// Construct a new ExtrinsicDomain.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `user_domain`.
///
/// See `user_domain` for correct usage and proof definition for this function.
///
/// # Arguments
/// * `identifier` - A string description of the data domain.
/// * `member` - A function used to test if a value is a member of the data domain.
/// * `descriptor` - Additional constraints on the domain.
#[no_mangle]
pub extern "C" fn opendp_internal___extrinsic_domain(
    identifier: *mut c_char,
    member: *const CallbackFn,
    descriptor: *mut ExtrinsicObject,
) -> FfiResult<*mut AnyDomain> {
    let identifier = try_!(to_str(identifier)).to_string();
    let descriptor = try_as_ref!(descriptor).clone();
    let member = wrap_func(try_as_ref!(member).clone());
    let element = ExtrinsicElement::new(identifier, descriptor);
    let member = Function::new_fallible(move |arg: &ExtrinsicObject| -> Fallible<bool> {
        member(&AnyObject::new(arg.clone()))?.downcast::<bool>()
    });

    Ok(AnyDomain::new(ExtrinsicDomain { element, member })).into()
}

#[bootstrap(
    name = "_extrinsic_divergence",
    arguments(descriptor(rust_type = "String"))
)]
/// Construct a new ExtrinsicDivergence, a privacy measure defined from a bindings language.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `user_divergence`.
///
/// See `user_divergence` for correct usage and proof definition for this function.
///
/// # Arguments
/// * `descriptor` - A string description of the privacy measure.
#[no_mangle]
pub extern "C" fn opendp_internal___extrinsic_divergence(
    descriptor: *mut c_char,
) -> FfiResult<*mut AnyMeasure> {
    let descriptor = try_!(to_str(descriptor)).to_string();
    Ok(AnyMeasure::new(ExtrinsicDivergence { descriptor })).into()
}

#[bootstrap(
    name = "_extrinsic_distance",
    arguments(descriptor(rust_type = "String"))
)]
/// Construct a new ExtrinsicDistance.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `user_distance`.
///
/// See `user_distance` for correct usage of this function.
///
/// # Arguments
/// * `descriptor` - A string description of the metric.
#[no_mangle]
pub extern "C" fn opendp_internal___extrinsic_distance(
    descriptor: *mut c_char,
) -> FfiResult<*mut AnyMetric> {
    let descriptor = try_!(to_str(descriptor)).to_string();
    Ok(AnyMetric::new(ExtrinsicDistance { descriptor })).into()
}

#[bootstrap(
    features("contrib"),
    arguments(function(rust_type = "$pass_through(TO)"))
)]
/// Construct a Function from a user-defined callback.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `new_function`.
///
/// See `new_function` for correct usage and proof definition for this function.
///
/// # Arguments
/// * `function` - A function mapping data to a value of type `TO`
///
/// # Generics
/// * `TO` - Output Type
#[allow(dead_code)]
fn _new_pure_function<TO>(function: CallbackFn) -> Fallible<AnyFunction> {
    let _ = function;
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_internal___new_pure_function(
    function: *const CallbackFn,
    TO: *const c_char,
) -> FfiResult<*mut AnyFunction> {
    let _TO = TO;
    FfiResult::Ok(util::into_raw(Function::new_fallible(wrap_func(
        try_as_ref!(function).clone(),
    ))))
}
