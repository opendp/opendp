use std::{fmt, ptr};
use std::ffi::{c_void, CStr};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_char;

use crate::{try_, try_as_ref};
use crate::error::{Error, ErrorVariant, ExplainUnwrap, Fallible};
use crate::ffi::any::{AnyMeasurement, AnyObject, AnyTransformation, IntoAnyMeasurementExt, IntoAnyTransformationExt};
use crate::ffi::util;
use crate::ffi::util::{c_bool, into_c_char_p};

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
        unsafe { CStr::from_ptr(self.variant).to_str().unwrap_or("Couldn't get variant!") }
    }

    fn message_str(&self) -> Option<&str> {
        unsafe { self.message.as_ref().map(|s| CStr::from_ptr(s).to_str().unwrap_or("Couldn't get message!")) }
    }
}

impl From<Error> for FfiError {
    fn from(mut error: Error) -> Self {
        Self {
            variant: try_!(util::into_c_char_p(format!("{:?}", error.variant))),
            message: try_!(error.message.map_or(Ok(ptr::null::<c_char>() as *mut c_char), util::into_c_char_p)),
            backtrace: try_!(util::into_c_char_p(if let ErrorVariant::RelationDebug = error.variant{
                String::default()
            } else {
                error.backtrace.resolve();
                format!("{:?}", error.backtrace)
            })),
        }
    }
}

impl Drop for FfiError {
    fn drop(&mut self) {
        let _variant = util::into_string(self.variant).unwrap_assert("variants do not contain null bytes");
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
        write!(f, "FfiError: {{ type: {}, message: {:?} }}", self.variant_str(), self.message_str())
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
            |v| Self::Ok(util::into_raw(TO::from(v))))
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
            _ => false
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

#[cfg(test)]
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
            unknown => return err!(NotImplemented, "Unknown ErrorVariant {}", unknown)
        };
        Error {
            variant,
            message: util::to_option_str(val.message).unwrap_test().map(|s| s.to_owned()),
            backtrace: backtrace::Backtrace::new_unresolved(),
        }
    }
}

#[cfg(test)]
impl<T> From<FfiResult<*mut T>> for Fallible<T> {
    fn from(result: FfiResult<*mut T>) -> Self {
        match result {
            FfiResult::Ok(ok) => Ok(util::into_owned(ok)?),
            FfiResult::Err(err) => Err(util::into_owned(err)?.into()),
        }
    }
}

#[no_mangle]
#[must_use]
pub extern "C" fn opendp_core___error_free(this: *mut FfiError) -> bool {
    util::into_owned(this).is_ok()
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_check(
    transformation: *const AnyTransformation,
    distance_in: *const AnyObject,
    distance_out: *const AnyObject,
) -> FfiResult<*mut c_bool> {
    let transformation = try_as_ref!(transformation);
    let distance_in = try_as_ref!(distance_in);
    let distance_out = try_as_ref!(distance_out);
    let status = try_!(transformation.check(&distance_in, &distance_out));
    FfiResult::Ok(util::into_raw(util::from_bool(status)))
}

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

#[no_mangle]
pub extern "C" fn opendp_core__measurement_invoke(this: *const AnyMeasurement, arg: *const AnyObject) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[no_mangle]
pub extern "C" fn opendp_core___measurement_free(this: *mut AnyMeasurement) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_invoke(this: *const AnyTransformation, arg: *const AnyObject) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[no_mangle]
pub extern "C" fn opendp_core___transformation_free(this: *mut AnyTransformation) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_carrier_type(this: *mut AnyTransformation) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.input_domain.carrier_type.descriptor.to_string())))
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_carrier_type(this: *mut AnyMeasurement) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.input_domain.carrier_type.descriptor.to_string())))
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_input_distance_type(this: *mut AnyTransformation) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.input_metric.distance_type.descriptor.to_string())))
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_output_distance_type(this: *mut AnyTransformation) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.output_metric.distance_type.descriptor.to_string())))
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_input_distance_type(this: *mut AnyMeasurement) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.input_metric.distance_type.descriptor.to_string())))
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_output_distance_type(this: *mut AnyMeasurement) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(this.output_measure.distance_type.descriptor.to_string())))
}


#[cfg(test)]
mod tests {
    use crate::core::{Function, Measurement, PrivacyRelation, Transformation};
    use crate::dist::{MaxDivergence, SymmetricDistance};
    use crate::dom::AllDomain;
    use crate::ffi::any::{Downcast, IntoAnyMeasurementExt, IntoAnyTransformationExt};
    use crate::ffi::util::ToCharP;
    use crate::traits::CheckNull;
    use crate::trans;

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

    // TODO: Find all the places we've duplicated this code and replace with common function.
    pub fn make_test_measurement<T: Clone + CheckNull>() -> Measurement<AllDomain<T>, AllDomain<T>, SymmetricDistance, MaxDivergence<f64>> {
        Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new(|arg: &T| arg.clone()),
            SymmetricDistance::default(),
            MaxDivergence::default(),
            PrivacyRelation::new(|_d_in, _d_out| true),
        )
    }

    // TODO: Find all the places we've duplicated this code and replace with common function.
    pub fn make_test_transformation<T: Clone + CheckNull>() -> Transformation<AllDomain<T>, AllDomain<T>, SymmetricDistance, SymmetricDistance> {
        trans::make_identity(AllDomain::<T>::new(), SymmetricDistance::default()).unwrap_test()
    }

    #[test]
    fn test_measurement_invoke() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>().into_any());
        let arg = AnyObject::new_raw(999);
        let res = opendp_core__measurement_invoke(measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_measurement_invoke_wrong_type() -> Fallible<()> {
        let measurement = util::into_raw(make_test_measurement::<i32>().into_any());
        let arg = AnyObject::new_raw(999.0);
        let res = Fallible::from(opendp_core__measurement_invoke(measurement, arg));
        assert_eq!(res.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }

    #[test]
    fn test_transformation_invoke() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>().into_any());
        let arg = AnyObject::new_raw(999);
        let res = opendp_core__transformation_invoke(transformation, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_transformation_invoke_wrong_type() -> Fallible<()> {
        let transformation = util::into_raw(make_test_transformation::<i32>().into_any());
        let arg = AnyObject::new_raw(999.0);
        let res = Fallible::from(opendp_core__transformation_invoke(transformation, arg));
        assert_eq!(res.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }
}
