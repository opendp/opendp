/// This code is used to generate a hidden `_internal` module in Python
/// containing APIs that allow for the construction of library primitives like Transformations and Measurements
/// that do not require the "honest-but-curious" flag to be set.
use std::{ffi::c_char, os::raw::c_void};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap, StabilityMap, Transformation},
    domains::ffi::opendp_domains__user_domain,
    error::Fallible,
    ffi::{
        any::{
            AnyDomain, AnyFunction, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject,
            AnyTransformation, CallbackFn, Downcast, wrap_func,
        },
        util::{self, ExtrinsicObject, Type, as_ref},
    },
    measures::ffi::opendp_measures__user_divergence,
    metrics::ffi::opendp_metrics__user_distance,
    utilities::{
        BinarySearchable, fallible_binary_search, fallible_exponential_bounds_search,
        signed_fallible_binary_search,
    },
};

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
) -> Fallible<Measurement<AnyDomain, AnyMetric, AnyMeasure, AnyObject>> {
    let _ = (
        input_domain,
        input_metric,
        output_measure,
        privacy_map,
        function,
    );
    panic!("this signature only exists for code generation")
}

#[unsafe(no_mangle)]
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
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_measure).clone(),
        Function::new_fallible(wrap_func(try_as_ref!(function).clone())),
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
#[unsafe(no_mangle)]
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
        try_as_ref!(input_metric).clone(),
        try_as_ref!(output_domain).clone(),
        try_as_ref!(output_metric).clone(),
        Function::new_fallible(wrap_func(try_as_ref!(function).clone())),
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
#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___extrinsic_domain(
    identifier: *mut c_char,
    member: *const CallbackFn,
    descriptor: *mut ExtrinsicObject,
) -> FfiResult<*mut AnyDomain> {
    opendp_domains__user_domain(identifier, member, descriptor)
}

#[bootstrap(
    name = "_extrinsic_divergence",
    arguments(
        identifier(c_type = "char *", rust_type = b"null"),
        descriptor(default = b"null", rust_type = "ExtrinsicObject")
    )
)]
/// Construct a new ExtrinsicDivergence, a privacy measure defined from a bindings language.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `user_divergence`.
///
/// See `user_divergence` for correct usage and proof definition for this function.
///
/// # Arguments
/// * `identifier` - A string description of the privacy measure.
/// * `descriptor` - Additional constraints on the privacy measure.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___extrinsic_divergence(
    identifier: *mut c_char,
    descriptor: *mut ExtrinsicObject,
) -> FfiResult<*mut AnyMeasure> {
    opendp_measures__user_divergence(identifier, descriptor)
}

#[bootstrap(
    name = "_extrinsic_distance",
    arguments(
        identifier(c_type = "char *", rust_type = b"null"),
        descriptor(default = b"null", rust_type = "ExtrinsicObject")
    )
)]
/// Construct a new ExtrinsicDistance.
/// This is meant for internal use, as it does not require "honest-but-curious",
/// unlike `user_distance`.
///
/// See `user_distance` for correct usage of this function.
///
/// # Arguments
/// * `identifier` - A string description of the metric.
/// * `descriptor` - Additional constraints on the domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___extrinsic_distance(
    identifier: *mut c_char,
    descriptor: *mut ExtrinsicObject,
) -> FfiResult<*mut AnyMetric> {
    opendp_metrics__user_distance(identifier, descriptor)
}

#[bootstrap(
    features("contrib"),
    arguments(function(rust_type = "$pass_through(TO)")),
    generics(TO(default = "ExtrinsicObject"))
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

#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___new_pure_function(
    function: *const CallbackFn,
    TO: *const c_char,
) -> FfiResult<*mut AnyFunction> {
    let _TO = TO;
    FfiResult::Ok(util::into_raw(Function::new_fallible(wrap_func(
        try_as_ref!(function).clone(),
    ))))
}

fn wrap_search_predicate<T>(predicate: *const CallbackFn) -> Fallible<impl Fn(&T) -> Fallible<bool>>
where
    T: Clone + 'static + Send + Sync,
{
    let predicate = wrap_func(try_as_ref!(predicate).clone());
    Ok(move |arg: &T| predicate(&AnyObject::new(arg.clone()))?.downcast::<bool>())
}

#[bootstrap(
    name = "_binary_search",
    arguments(
        predicate(rust_type = "bool"),
        lower(default = b"null", c_type = "void *"),
        upper(default = b"null", c_type = "void *"),
        return_sign(default = false),
    ),
    generics(T(example = "$get_first_non_null(lower, upper)"))
)]
/// Find the closest passing value to the decision boundary of `predicate`.
///
/// This is meant for internal use by the Python and R bindings.
///
/// # Arguments
/// * `predicate` - A monotonic unary function from a number to a boolean.
/// * `lower` - Optional lower bound on the input to `predicate`.
/// * `upper` - Optional upper bound on the input to `predicate`.
/// * `return_sign` - If true, also return the direction away from the decision boundary.
///
/// # Generics
/// * `T` - Search type.
#[allow(dead_code)]
fn _binary_search<T>(
    predicate: CallbackFn,
    lower: Option<T>,
    upper: Option<T>,
    return_sign: bool,
) -> Fallible<AnyObject> {
    let _ = (predicate, lower, upper, return_sign);
    panic!("this signature only exists for code generation")
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___binary_search(
    predicate: *const CallbackFn,
    lower: *const c_void,
    upper: *const c_void,
    return_sign: bool,
    T: *const c_char,
) -> FfiResult<*mut AnyObject> {
    fn monomorphize<T>(
        predicate: *const CallbackFn,
        lower: *const c_void,
        upper: *const c_void,
        return_sign: bool,
    ) -> Fallible<AnyObject>
    where
        T: BinarySearchable + 'static + Send + Sync,
    {
        let predicate = wrap_search_predicate::<T>(predicate)?;
        let lower = as_ref(lower as *const T).cloned();
        let upper = as_ref(upper as *const T).cloned();
        let bounds = (lower, upper);

        if return_sign {
            signed_fallible_binary_search(predicate, bounds)
                .map(|(value, sign)| AnyObject::new((value, i32::from(sign))))
        } else {
            fallible_binary_search(predicate, bounds).map(AnyObject::new)
        }
    }

    let T = try_!(Type::try_from(T));
    dispatch!(
        monomorphize,
        [(
            T,
            [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64]
        )],
        (predicate, lower, upper, return_sign)
    )
    .into()
}

#[bootstrap(
    name = "_exponential_bounds_search",
    arguments(predicate(rust_type = "bool"))
)]
/// Determine bounds for a binary search via an exponential search.
///
/// This is meant for internal use by the Python and R bindings.
///
/// # Arguments
/// * `predicate` - A monotonic unary function from a number to a boolean.
///
/// # Generics
/// * `T` - Search type.
#[allow(dead_code)]
fn _exponential_bounds_search<T>(predicate: CallbackFn) -> Fallible<Option<(T, T)>> {
    let _ = predicate;
    panic!("this signature only exists for code generation")
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___exponential_bounds_search(
    predicate: *const CallbackFn,
    T: *const c_char,
) -> FfiResult<*mut AnyObject> {
    fn monomorphize<T>(predicate: *const CallbackFn) -> Fallible<AnyObject>
    where
        T: BinarySearchable + 'static + Send + Sync,
    {
        let predicate = wrap_search_predicate::<T>(predicate)?;
        fallible_exponential_bounds_search(&predicate).map(AnyObject::new)
    }

    let T = try_!(Type::try_from(T));
    dispatch!(
        monomorphize,
        [(
            T,
            [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64]
        )],
        (predicate)
    )
    .into()
}

#[bootstrap(
    generics(T(default = "float")),
    arguments(max_depth(c_type = "size_t")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Bound the Lp sensitivity of floating-point sums, including rounding error.
/// Assumes finite summands and round-to-nearest addition with gradual underflow.
///
/// Let a_j be the exact sum of the absolute values of the summands in coordinate j,
/// on either input. The norm bounds apply to a, not to the vector of final sums.
///
/// # Arguments
/// * `sensitivity` - Lp sensitivity in exact arithmetic (p >= 1).
/// * `max_depth` - Upper bound on summation tree depth.
/// * `l_inf_bound` - Upper bound on the L-infinity norm of a.
/// * `lp_bound` - Upper bound on the Lp norm of a.
/// # Generics
/// * `T` - Summation type (f32 or f64).
fn _float_sum_sensitivity<T: crate::traits::Float>(
    sensitivity: f64,
    max_depth: usize,
    l_inf_bound: f64,
    lp_bound: f64,
) -> Fallible<f64> {
    crate::traits::sum::float_sum_sensitivity::<T>(sensitivity, max_depth, l_inf_bound, lp_bound)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___float_sum_sensitivity(
    sensitivity: f64,
    max_depth: usize,
    l_inf_bound: f64,
    lp_bound: f64,
    T: *const c_char,
) -> FfiResult<*mut AnyObject> {
    fn monomorphize<T: crate::traits::Float>(
        sensitivity: f64,
        max_depth: usize,
        l_inf_bound: f64,
        lp_bound: f64,
    ) -> FfiResult<*mut AnyObject> {
        _float_sum_sensitivity::<T>(sensitivity, max_depth, l_inf_bound, lp_bound)
            .map(AnyObject::new)
            .into()
    }

    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T, @floats)
    ], (sensitivity, max_depth, l_inf_bound, lp_bound))
}

#[bootstrap(
    arguments(p(c_type = "size_t")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Bound an Lp norm from L0, L1 and L-infinity bounds.
///
/// # Arguments
/// * `distance` - (L0, L1, L-infinity) bounds.
/// * `p` - 1 or 2.
fn _l01inf_to_lp(distance: (u32, u32, u32), p: usize) -> Fallible<f64> {
    crate::metrics::l01inf_to_lp(distance, p)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_internal___l01inf_to_lp(
    distance: *const AnyObject,
    p: usize,
) -> FfiResult<*mut AnyObject> {
    let distance = *try_!(try_as_ref!(distance).downcast_ref::<(u32, u32, u32)>());
    _l01inf_to_lp(distance, p).map(AnyObject::new).into()
}
