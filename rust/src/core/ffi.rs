use std::ffi::{c_void, CStr};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_char;
use std::{fmt, ptr};

use opendp_derive::bootstrap;

use crate::error::{Error, ErrorVariant, ExplainUnwrap, Fallible};
use crate::ffi::any::{
    wrap_func, AnyDomain, AnyFunction, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject,
    AnyQueryable, AnyTransformation, CallbackFn, Downcast, QueryType,
};
use crate::ffi::util::into_c_char_p;
use crate::ffi::util::{self, c_bool, Type};
use crate::interactive::{Answer, Query, Queryable};
use crate::{try_, try_as_ref};

use super::{Domain, Function, Measure, Measurement, Metric, MetricSpace, Transformation};

#[repr(C)]
pub struct FfiSlice {
    pub ptr: *const c_void,
    pub len: usize,
}

impl FfiSlice {
    pub fn new(ptr: *mut c_void, len: usize) -> Self {
        Self { ptr, len }
    }
}

#[repr(C)]
pub struct FfiError {
    pub variant: *mut c_char,
    pub message: *mut c_char,
    // MAY BE NULL!
    pub backtrace: *mut c_char,
}

impl FfiError {
    fn variant_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.variant)
                .to_str()
                .unwrap_or("Couldn't get variant!")
        }
    }

    fn message_str(&self) -> Option<&str> {
        unsafe {
            self.message.as_ref().map(|s| {
                CStr::from_ptr(s)
                    .to_str()
                    .unwrap_or("Couldn't get message!")
            })
        }
    }
}

impl From<Error> for FfiError {
    fn from(error: Error) -> Self {
        Self {
            variant: try_!(util::into_c_char_p(format!("{:?}", error.variant))),
            message: try_!(error.message.map_or(
                Ok(ptr::null::<c_char>() as *mut c_char),
                util::into_c_char_p
            )),
            backtrace: try_!(util::into_c_char_p(error.backtrace.to_string())),
        }
    }
}

impl Drop for FfiError {
    fn drop(&mut self) {
        let _variant =
            util::into_string(self.variant).unwrap_assert("variants do not contain null bytes");
        let _message = unsafe { self.message.as_mut() }.map(|p| util::into_string(p).unwrap());
        let _backtrace = util::into_string(self.backtrace).unwrap();
    }
}

impl PartialEq for FfiError {
    fn eq(&self, other: &Self) -> bool {
        self.variant_str() == other.variant_str() && self.message_str() == other.message_str()
    }
}

impl Debug for FfiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FfiError: {{ type: {}, message: {:?} }}",
            self.variant_str(),
            self.message_str()
        )
    }
}

// Using this repr means we'll get a tagged union in C.
// Because this is a generic, we need to be careful about sizes. Currently, everything that goes in here
// is a pointer, so we're OK, but we may need to revisit this.
#[repr(C, u32)]
pub enum FfiResult<T> {
    Ok(T),
    Err(*mut FfiError),
}

impl<TI, TO: From<TI>> From<Fallible<TI>> for FfiResult<*mut TO> {
    fn from(result: Fallible<TI>) -> Self {
        result.map_or_else(
            |e| Self::Err(util::into_raw(FfiError::from(e))),
            |v| Self::Ok(util::into_raw(TO::from(v))),
        )
    }
}

impl<T> From<Error> for FfiResult<T> {
    fn from(e: Error) -> Self {
        Self::Err(util::into_raw(FfiError::from(e)))
    }
}

impl<T: PartialEq> PartialEq for FfiResult<*mut T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ok(self_), Self::Ok(other)) => util::as_ref(*self_) == util::as_ref(*other),
            (Self::Err(self_), Self::Err(other)) => util::as_ref(*self_) == util::as_ref(*other),
            _ => false,
        }
    }
}

impl<T: Debug> Debug for FfiResult<*mut T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FfiResult::Ok(ok) => write!(f, "Ok({:?})", util::as_ref(*ok).unwrap_test()),
            FfiResult::Err(err) => write!(f, "Err({:?})", util::as_ref(*err).unwrap_test()),
        }
    }
}

/// Trait to convert Result<Measurement> into FfiResult<*mut AnyMeasurement>. We can't do this with From
/// because there's a blanket implementation of From for FfiResult. We can't do this with a method on Result
/// because it comes from another crate. So we need a separate trait.
pub trait IntoAnyMeasurementFfiResultExt {
    fn into_any(self) -> Fallible<AnyMeasurement>;
}

impl<DI: 'static + Domain, TO: 'static, MI: 'static + Metric, MO: 'static + Measure>
    IntoAnyMeasurementFfiResultExt for Fallible<Measurement<DI, TO, MI, MO>>
where
    MO::Distance: 'static,
    (DI, MI): MetricSpace,
{
    fn into_any(self) -> Fallible<AnyMeasurement> {
        self.map(Measurement::into_any)
    }
}

/// Trait to convert Result<Transformation> into FfiResult<*mut AnyTransformation>. We can't do this with From
/// because there's a blanket implementation of From for FfiResult. We can't do this with a method on Result
/// because it comes from another crate. So we need a separate trait.
pub trait IntoAnyTransformationFfiResultExt {
    fn into_any(self) -> Fallible<AnyTransformation>;
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Metric>
    IntoAnyTransformationFfiResultExt for Fallible<Transformation<DI, DO, MI, MO>>
where
    DO::Carrier: 'static,
    MO::Distance: 'static,
    (DI, MI): MetricSpace,
    (DO, MO): MetricSpace,
{
    fn into_any(self) -> Fallible<AnyTransformation> {
        self.map(Transformation::into_any)
    }
}

pub trait IntoAnyFunctionFfiResultExt {
    fn into_any(self) -> Fallible<AnyFunction>;
}

impl<TI: 'static, TO: 'static> IntoAnyFunctionFfiResultExt for Fallible<Function<TI, TO>> {
    fn into_any(self) -> Fallible<AnyFunction> {
        self.map(Function::into_any)
    }
}

impl From<FfiError> for Error {
    fn from(val: FfiError) -> Self {
        let variant = util::to_str(val.variant).unwrap_assert("variants do not contain null bytes");
        let variant = match variant {
            "FFI" => ErrorVariant::FFI,
            "TypeParse" => ErrorVariant::TypeParse,
            "FailedFunction" => ErrorVariant::FailedFunction,
            "FailedMap" => ErrorVariant::FailedMap,
            "RelationDebug" => ErrorVariant::RelationDebug,
            "FailedCast" => ErrorVariant::FailedCast,
            "DomainMismatch" => ErrorVariant::DomainMismatch,
            "MakeTransformation" => ErrorVariant::MakeTransformation,
            "MakeMeasurement" => ErrorVariant::MakeMeasurement,
            "InvalidDistance" => ErrorVariant::InvalidDistance,
            "NotImplemented" => ErrorVariant::NotImplemented,
            unknown => return err!(NotImplemented, "Unknown ErrorVariant {}", unknown),
        };
        Error {
            variant,
            message: util::to_option_str(val.message)
                .unwrap_test()
                .map(|s| s.to_owned()),
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

impl<T> From<FfiResult<*mut T>> for Fallible<T> {
    fn from(result: FfiResult<*mut T>) -> Self {
        match result {
            FfiResult::Ok(ok) => Ok(util::into_owned(ok)?),
            FfiResult::Err(err) => Err(util::into_owned(err)?.into()),
        }
    }
}

#[bootstrap(
    name = "_error_free",
    arguments(this(c_type = "FfiError *", do_not_convert = true, hint = "FfiError"))
)]
/// Internal function. Free the memory associated with `error`.
///
/// # Returns
/// A boolean, where true indicates successful free
#[no_mangle]
#[must_use]
pub extern "C" fn opendp_core___error_free(this: *mut FfiError) -> bool {
    util::into_owned(this).is_ok()
}

#[bootstrap(
    name = "transformation_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>", do_not_convert = true)
)]
/// Get the input domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_domain(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "transformation_output_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>", do_not_convert = true)
)]
/// Get the output domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_output_domain(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_domain.clone()))
}

#[bootstrap(
    name = "transformation_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>", do_not_convert = true)
)]
/// Get the input domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_metric(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "transformation_output_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>", do_not_convert = true)
)]
/// Get the output domain from a `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_output_metric(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_metric.clone()))
}

#[bootstrap(
    name = "measurement_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>", do_not_convert = true)
)]
/// Get the input domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_domain(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "measurement_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>", do_not_convert = true)
)]
/// Get the input domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_metric(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "measurement_output_measure",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMeasure *>", do_not_convert = true)
)]
/// Get the output domain from a `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_output_measure(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyMeasure> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_measure.clone()))
}

#[bootstrap(
    name = "measurement_function",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyFunction *>", do_not_convert = true)
)]
/// Get the function from a measurement.
///
/// # Arguments
/// * `this` - The measurement to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_function(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut AnyFunction> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).function.clone()))
}

#[bootstrap(
    name = "transformation_map",
    arguments(
        transformation(rust_type = b"null"),
        distance_in(rust_type = "$transformation_input_distance_type(transformation)")
    )
)]
/// Use the `transformation` to map a given `d_in` to `d_out`.
///
/// # Arguments
/// * `transformation` - Transformation to check the map distances with.
/// * `distance_in` - Distance in terms of the input metric.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_map(
    transformation: *const AnyTransformation,
    distance_in: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let transformation = try_as_ref!(transformation);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = transformation.map(distance_in);
    distance_out.into()
}

#[bootstrap(
    name = "transformation_check",
    arguments(
        transformation(rust_type = b"null"),
        distance_in(rust_type = "$transformation_input_distance_type(transformation)"),
        distance_out(rust_type = "$transformation_output_distance_type(transformation)"),
    ),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
///
/// # Arguments
/// * `measurement` - Measurement to check the privacy relation of.
/// * `d_in` - Distance in terms of the input metric.
/// * `d_out` - Distance in terms of the output metric.
///
/// # Returns
/// True indicates that the relation passed at the given distance.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_check(
    transformation: *const AnyTransformation,
    distance_in: *const AnyObject,
    distance_out: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let transformation = try_as_ref!(transformation);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = try_as_ref!(distance_out);
    let status = try_!(transformation.check(distance_in, distance_out));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "measurement_map",
    arguments(
        measurement(rust_type = b"null"),
        distance_in(rust_type = "$measurement_input_distance_type(measurement)"),
        distance_out(rust_type = "$measurement_output_distance_type(measurement)"),
    )
)]
/// Use the `measurement` to map a given `d_in` to `d_out`.
///
/// # Arguments
/// * `measurement` - Measurement to check the map distances with.
/// * `distance_in` - Distance in terms of the input metric.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_map(
    measurement: *const AnyMeasurement,
    distance_in: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let measurement = try_as_ref!(measurement);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = measurement.map(distance_in);
    distance_out.into()
}

#[bootstrap(
    name = "measurement_check",
    arguments(
        measurement(rust_type = b"null"),
        distance_in(rust_type = "$measurement_input_distance_type(measurement)"),
        distance_out(rust_type = "$measurement_output_distance_type(measurement)"),
    ),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
///
/// # Arguments
/// * `measurement` - Measurement to check the privacy relation of.
/// * `d_in` - Distance in terms of the input metric.
/// * `d_out` - Distance in terms of the output metric.
///
/// # Returns
/// True indicates that the relation passed at the given distance.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_check(
    measurement: *const AnyMeasurement,
    distance_in: *const AnyObject,
    distance_out: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let measurement = try_as_ref!(measurement);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = try_as_ref!(distance_out);
    let status = try_!(measurement.check(distance_in, distance_out));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

#[bootstrap(
    name = "measurement_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$measurement_input_carrier_type(this)")
    )
)]
/// Invoke the `measurement` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Measurement to invoke.
/// * `arg` - Input data to supply to the measurement. A member of the measurement's input domain.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_invoke(
    this: *const AnyMeasurement,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "_measurement_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___measurement_free(this: *mut AnyMeasurement) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "transformation_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$transformation_input_carrier_type(this)")
    )
)]
/// Invoke the `transformation` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Transformation to invoke.
/// * `arg` - Input data to supply to the transformation. A member of the transformation's input domain.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_invoke(
    this: *const AnyTransformation,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "transformation_function",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyFunction *>", do_not_convert = true)
)]
/// Get the function from a transformation.
///
/// # Arguments
/// * `this` - The transformation to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_function(
    this: *mut AnyTransformation,
) -> FfiResult<*mut AnyFunction> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).function.clone()))
}

#[bootstrap(
    name = "_transformation_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___transformation_free(
    this: *mut AnyTransformation,
) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "transformation_input_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input (carrier) data type of `this`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_carrier_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_domain.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_input_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input (carrier) data type of `this`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_carrier_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_domain.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "transformation_input_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input distance type of `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_distance_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "transformation_output_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the output distance type of `transformation`.
///
/// # Arguments
/// * `this` - The transformation to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__transformation_output_distance_type(
    this: *mut AnyTransformation,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.output_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_input_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input distance type of `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_distance_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_metric.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "measurement_output_distance_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the output distance type of `measurement`.
///
/// # Arguments
/// * `this` - The measurement to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__measurement_output_distance_type(
    this: *mut AnyMeasurement,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.output_measure.distance_type.descriptor.to_string()
    )))
}

#[bootstrap(
    features("contrib", "honest-but-curious"),
    arguments(function(rust_type = "$pass_through(TO)")),
    dependencies("c_function")
)]
/// Construct a Function from a user-defined callback.
/// Can be used to build a post-processor.
///
/// # Why honest-but-curious?
/// An OpenDP `function` must satisfy two criteria.
/// These invariants about functions are necessary to show correctness of other algorithms.
///
/// First, `function` must not use global state.
/// For instance, a postprocessor that accesses the system clock time
/// can be used to build a measurement that reveals elapsed execution time,
/// which escalates a side-channel vulnerability into a direct vulnerability.
///
/// Secondly, `function` must only raise data-independent exceptions.
/// For instance, raising an exception with the value of a DP release will both
/// reveal the DP output and cancel the computation, potentially avoiding privacy accounting.
///
/// # Arguments
/// * `function` - A function mapping data to a value of type `TO`
///
/// # Generics
/// * `TO` - Output Type
#[allow(dead_code)]
fn new_function<TO>(function: *const CallbackFn) -> Fallible<AnyFunction> {
    let _ = function;
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_core__new_function(
    function: *const CallbackFn,
    TO: *const c_char,
) -> FfiResult<*mut AnyFunction> {
    let function = try_as_ref!(function).clone();
    let _TO = TO;
    FfiResult::Ok(util::into_raw(Function::new_fallible(wrap_func(function))))
}

#[bootstrap(
    name = "function_eval",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$parse_or_infer(TI, arg)"),
        TI(rust_type = b"null", default = b"null"),
    )
)]
/// Eval the `function` with `arg`.
///
/// # Arguments
/// * `this` - Function to invoke.
/// * `arg` - Input data to supply to the measurement. A member of the measurement's input domain.
/// * `TI` - Input Type.
#[no_mangle]
pub extern "C" fn opendp_core__function_eval(
    this: *const AnyFunction,
    arg: *const AnyObject,
    TI: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    let _TI = TI;
    this.eval(arg).into()
}

#[bootstrap(
    name = "_function_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___function_free(this: *mut AnyFunction) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "queryable_eval",
    arguments(
        queryable(rust_type = b"null"),
        query(rust_type = "$queryable_query_type(queryable)")
    )
)]
/// Invoke the `queryable` with `query`. Returns a differentially private release.
///
/// # Arguments
/// * `queryable` - Queryable to eval.
/// * `query` - Input data to supply to the measurement. A member of the measurement's input domain.
#[no_mangle]
pub extern "C" fn opendp_core__queryable_eval(
    queryable: *mut AnyObject,
    query: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_as_mut_ref!(queryable);
    let queryable = try_!(queryable.downcast_mut::<AnyQueryable>());
    let query = try_as_ref!(query);
    queryable.eval(query).into()
}

#[bootstrap(
    name = "queryable_query_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the query type of `queryable`.
///
/// # Arguments
/// * `this` - The queryable to retrieve the query type from.
#[no_mangle]
pub extern "C" fn opendp_core__queryable_query_type(
    this: *mut AnyObject,
) -> FfiResult<*mut c_char> {
    let this = try_as_mut_ref!(this);
    let this = try_!(this.downcast_mut::<AnyQueryable>());
    let answer: Type = try_!(this.eval_internal(&QueryType));
    FfiResult::Ok(try_!(into_c_char_p(answer.descriptor.to_string())))
}

type TransitionFn = extern "C" fn(*const AnyObject, c_bool) -> *mut FfiResult<*mut AnyObject>;

// wrap a TransitionFn in a closure, so that it can be used in Queryables
fn wrap_transition(
    transition: TransitionFn,
    Q: Type,
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
    name = "new_queryable",
    features("contrib"),
    arguments(transition(rust_type = "$pass_through(A)")),
    generics(Q(default = "ExtrinsicObject"), A(default = "ExtrinsicObject")),
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
fn new_queryable<Q, A>(transition: TransitionFn) -> Fallible<AnyObject> {
    let _ = transition;
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_core__new_queryable(
    transition: TransitionFn,
    Q: *const c_char,
    A: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let Q = try_!(Type::try_from(Q));
    let _A = A;
    FfiResult::Ok(util::into_raw(AnyObject::new(try_!(Queryable::new(
        wrap_transition(transition, Q)
    )))))
}

#[cfg(test)]
mod tests {
    use crate::combinators::test::{make_test_measurement, make_test_transformation};
    use crate::ffi::any::Downcast;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_ffi_error_from_error() {
        let err = err!(FailedFunction, "Eat my shorts!");
        let ffi_err: FfiError = err.into();
        assert_eq!(
            ffi_err,
            FfiError {
                variant: "FailedFunction".to_char_p(),
                message: "Eat my shorts!".to_char_p(),
                backtrace: "".to_char_p(),
            }
        )
    }

    #[test]
    fn test_ffi_result_from_result_ok() {
        let res = Ok(999);
        let ffi_res = FfiResult::from(res);
        assert_eq!(FfiResult::Ok(util::into_raw(999)), ffi_res);
    }

    #[test]
    fn test_ffi_result_from_result_err() {
        let res: Fallible<i32> = fallible!(FailedFunction, "Eat my shorts!");
        let ffi_res: FfiResult<*mut i32> = FfiResult::from(res);
        assert_eq!(
            ffi_res,
            FfiResult::Err(util::into_raw(FfiError {
                variant: "FailedFunction".to_char_p(),
                message: "Eat my shorts!".to_char_p(),
                backtrace: "".to_char_p(),
            }))
        );
    }

    #[test]
    fn test_error_from_ffi_error() {
        let ffi_err = FfiError {
            variant: "FailedFunction".to_char_p(),
            message: "Eat my shorts!".to_char_p(),
            backtrace: "".to_char_p(),
        };
        let err: Error = ffi_err.into();
        assert_eq!(err, err!(FailedFunction, "Eat my shorts!"))
    }

    #[test]
    fn test_result_from_ffi_result_ok() {
        let ffi_res = FfiResult::Ok(util::into_raw(123));
        let res = Fallible::from(ffi_res);
        assert_eq!(res, Ok(123));
    }

    #[test]
    fn test_result_from_ffi_result_err() {
        let ffi_res: FfiResult<*mut i32> = FfiResult::Err(util::into_raw(FfiError {
            variant: "FailedFunction".to_char_p(),
            message: "Eat my shorts!".to_char_p(),
            backtrace: "".to_char_p(),
        }));
        let res = Fallible::from(ffi_res);
        assert_eq!(res, fallible!(FailedFunction, "Eat my shorts!"));
    }

    #[test]
    fn test_measurement_invoke() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>()?.into_any());
        let arg = AnyObject::new_raw(vec![999]);
        let res = opendp_core__measurement_invoke(measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_measurement_invoke_wrong_type() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>()?.into_any());
        let arg = AnyObject::new_raw(vec![999.0]);
        let res = Fallible::from(opendp_core__measurement_invoke(measurement, arg));
        assert_eq!(res.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }

    #[test]
    fn test_transformation_invoke() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>()?.into_any());
        let arg = AnyObject::new_raw(vec![999]);
        let res = opendp_core__transformation_invoke(transformation, arg);
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![999]);
        Ok(())
    }

    #[test]
    fn test_transformation_invoke_wrong_type() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>()?.into_any());
        let arg = AnyObject::new_raw(999.0);
        let res = Fallible::from(opendp_core__transformation_invoke(transformation, arg));
        assert_eq!(res.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }
}
