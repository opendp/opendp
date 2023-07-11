use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap, StabilityMap, Transformation},
    error::Fallible,
    ffi::{
        any::{
            AnyDomain, AnyFunction, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyQueryable,
            AnyTransformation, QueryType,
        },
        util::{self, c_bool, Type},
    },
    interactive::{Answer, Query, Queryable},
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
    Measurement::new(
        try_as_ref!(input_domain).clone(),
        Function::new_fallible(wrap_func(function)),
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_measure).clone(),
        PrivacyMap::new_fallible(wrap_func(privacy_map)),
    )
    .into()
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

type TransitionFn = extern "C" fn(*const AnyObject, c_bool) -> *mut FfiResult<*mut AnyObject>;

// wrap a TransitionFn in a closure, so that it can be used in Queryables
fn wrap_trans(
    transition: TransitionFn,
    Q: Type
) -> impl FnMut(&AnyQueryable, Query<AnyObject>) -> Fallible<Answer<AnyObject>> {
    fn eval(transition: &TransitionFn, q: &AnyObject, is_internal: bool) -> Fallible<AnyObject> {
        util::into_owned(transition(
            q as *const AnyObject,
            util::from_bool(is_internal),
        ))?
        .into()
    }

    move |_self: &AnyQueryable, arg: Query<AnyObject>| -> Fallible<Answer<AnyObject>> {
        Ok(match arg {
            Query::External(q) => Answer::External(eval(&transition, q, false)?),
            Query::Internal(q) => {
                if q.downcast_ref::<QueryType>().is_some() {
                    return Ok(Answer::internal(Q.clone()));
                }
                let q = q
                    .downcast_ref::<AnyObject>()
                    .ok_or_else(|| err!(FFI, "failed to downcast internal query to AnyObject"))?;

                Answer::Internal(Box::new(eval(&transition, q, true)?))
            }
        })
    }
}

#[bootstrap(
    name = "new_user_queryable",
    features("contrib"),
    arguments(transition(rust_type = "$pass_through(A)")),
    dependencies("c_transition")
)]
/// Construct a queryable from a user-defined transition function.
///
/// # Arguments
/// * `transition` - A transition function taking a reference to self, a query, and an internal/external indicator
///
/// # Generics
/// * `Q` - Query Type
/// * `A` - Output Type
#[allow(dead_code)]
fn new_user_queryable<Q, A>(transition: TransitionFn) -> Fallible<AnyObject> {
    let _ = transition;
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_combinators__new_user_queryable(
    transition: TransitionFn,
    Q: *const c_char,
    A: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let Q = try_!(Type::try_from(Q));
    let _A = A;
    FfiResult::Ok(util::into_raw(AnyObject::new(try_!(Queryable::new(
        wrap_trans(transition, Q)
    )))))
}
