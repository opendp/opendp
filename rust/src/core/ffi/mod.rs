use std::ffi::{CStr, c_void};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_char;
use std::{fmt, ptr};

use opendp_derive::bootstrap;

use crate::error::{Error, ErrorVariant, ExplainUnwrap, Fallible};
use crate::ffi::any::{AnyFunction, AnyObject, CallbackFn, wrap_func};
use crate::ffi::util;
use crate::{try_, try_as_ref};

mod measurement;
pub use measurement::*;
mod odometer;
pub use odometer::*;
mod transformation;
pub use transformation::*;
mod queryable;
pub use queryable::*;

use super::Function;

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

pub trait IntoAnyFunctionFfiResultExt {
    fn into_any(self) -> Fallible<AnyFunction>;
}

impl<TI: 'static, TO: 'static + Send + Sync> IntoAnyFunctionFfiResultExt
    for Fallible<Function<TI, TO>>
{
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
            "MetricMismatch" => ErrorVariant::MetricMismatch,
            "MeasureMismatch" => ErrorVariant::MeasureMismatch,
            "MakeDomain" => ErrorVariant::MakeDomain,
            "MakeTransformation" => ErrorVariant::MakeTransformation,
            "MakeMeasurement" => ErrorVariant::MakeMeasurement,
            "MetricSpace" => ErrorVariant::MetricSpace,
            "InvalidDistance" => ErrorVariant::InvalidDistance,
            "Overflow" => ErrorVariant::Overflow,
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
#[unsafe(no_mangle)]
#[must_use]
pub extern "C" fn opendp_core___error_free(this: *mut FfiError) -> bool {
    util::into_owned(this).is_ok()
}

#[bootstrap(
    rust_path = "core/struct.Function",
    features("contrib", "honest-but-curious"),
    arguments(function(rust_type = "$pass_through(TO)"))
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

#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core___function_free(this: *mut AnyFunction) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[cfg(test)]
mod tests {
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
}
