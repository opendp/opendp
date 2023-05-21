use std::ffi::{c_void, CStr};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_char;
use std::{fmt, ptr};

use opendp_derive::bootstrap;

use crate::error::{Error, ErrorVariant, ExplainUnwrap, Fallible};
use crate::ffi::any::{
    AnyFunction, AnyMeasurement, AnyObject, AnyQueryable, AnyTransformation, Downcast,
    IntoAnyFunctionExt, IntoAnyMeasurementExt, IntoAnyTransformationExt, QueryOdometerInvokeType,
    QueryOdometerMapType, QueryType,
};
use crate::ffi::util::into_c_char_p;
use crate::ffi::util::{self, Type};
use crate::{try_, try_as_ref};

mod measurement;
pub use measurement::*;
mod odometer;
pub use odometer::*;
mod transformation;
pub use transformation::*;

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
    fn into_any(self) -> FfiResult<*mut AnyMeasurement>;
}

impl<T: IntoAnyMeasurementExt> IntoAnyMeasurementFfiResultExt for Fallible<T> {
    fn into_any(self) -> FfiResult<*mut AnyMeasurement> {
        self.map(IntoAnyMeasurementExt::into_any).into()
    }
}

/// Trait to convert Result<Transformation> into FfiResult<*mut AnyTransformation>. We can't do this with From
/// because there's a blanket implementation of From for FfiResult. We can't do this with a method on Result
/// because it comes from another crate. So we need a separate trait.
pub trait IntoAnyTransformationFfiResultExt {
    fn into_any(self) -> FfiResult<*mut AnyTransformation>;
}

impl<T: IntoAnyTransformationExt> IntoAnyTransformationFfiResultExt for Fallible<T> {
    fn into_any(self) -> FfiResult<*mut AnyTransformation> {
        self.map(IntoAnyTransformationExt::into_any).into()
    }
}

pub trait IntoAnyFunctionFfiResultExt {
    fn into_any(self) -> FfiResult<*mut AnyFunction>;
}

impl<T: IntoAnyFunctionExt> IntoAnyFunctionFfiResultExt for Fallible<T> {
    fn into_any(self) -> FfiResult<*mut AnyFunction> {
        self.map(IntoAnyFunctionExt::into_any).into()
    }
}

impl From<FfiError> for Error {
    fn from(val: FfiError) -> Self {
        let variant = util::to_str(val.variant).unwrap_assert("variants do not contain null bytes");
        let variant = match variant {
            "FFI" => ErrorVariant::FFI,
            "TypeParse" => ErrorVariant::TypeParse,
            "FailedFunction" => ErrorVariant::FailedFunction,
            "FailedRelation" => ErrorVariant::FailedRelation,
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
/// Eval the `queryable` with `query`. Returns a differentially private release.
///
/// # Arguments
/// * `queryable` - Queryable to eval.
/// * `query` - The input to the queryable.
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

#[bootstrap(
    name = "queryable_query_odometer_invoke_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the query odometer invoke type of `queryable`.
///
/// # Arguments
/// * `this` - The queryable to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__queryable_query_odometer_invoke_type(
    this: *mut AnyObject,
) -> FfiResult<*mut c_char> {
    let this = try_as_mut_ref!(this);
    let this = try_!(this.downcast_mut::<AnyQueryable>());
    let answer: Type = try_!(this.eval_internal(&QueryOdometerInvokeType));
    FfiResult::Ok(try_!(into_c_char_p(answer.descriptor.to_string())))
}

#[bootstrap(
    name = "queryable_query_odometer_map_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the query odometer map type of `queryable`.
///
/// # Arguments
/// * `this` - The queryable to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__queryable_query_odometer_map_type(
    this: *mut AnyObject,
) -> FfiResult<*mut c_char> {
    let this = try_as_mut_ref!(this);
    let this = try_!(this.downcast_mut::<AnyQueryable>());
    let answer: Type = try_!(this.eval_internal(&QueryOdometerMapType));
    FfiResult::Ok(try_!(into_c_char_p(answer.descriptor.to_string())))
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
