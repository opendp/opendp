use std::convert::TryFrom;
use std::fmt::Debug;
use std::os::raw::{c_char, c_void};

use num::{Float, One, Zero};

use crate::accuracy::*;
use crate::core::{FfiError, FfiResult};
use crate::err;
use crate::ffi::any::AnyObject;
use crate::ffi::util;
use crate::ffi::util::Type;
use crate::traits::InfCast;

macro_rules! build_extern_accuracy {
    ($arg:ident, $ffi_func:ident, $func:ident) => {
        #[no_mangle]
        pub extern "C" fn $ffi_func(
            $arg: *const c_void,
            alpha: *const c_void,
            T: *const c_char,
        ) -> FfiResult<*mut AnyObject> {
            fn monomorphize<T>(
                $arg: *const c_void, alpha: *const c_void
            ) -> FfiResult<*mut AnyObject> where
                T: 'static + Float + One + Zero + Debug + InfCast<f64>,
                f64: InfCast<T> {
                let $arg = *try_as_ref!($arg as *const T);
                let alpha = *try_as_ref!(alpha as *const T);

                $func($arg, alpha).map_or_else(
                    |e| FfiResult::Err(util::into_raw(FfiError::from(e))),
                    |v| FfiResult::Ok(util::into_raw(AnyObject::new(v))))
            }
            let T = try_!(Type::try_from(T));
            dispatch!(monomorphize, [
                (T, @floats)
            ], ($arg, alpha))
        }
    }
}

build_extern_accuracy!(scale, opendp_accuracy__laplacian_scale_to_accuracy, laplacian_scale_to_accuracy);
build_extern_accuracy!(accuracy, opendp_accuracy__accuracy_to_laplacian_scale, accuracy_to_laplacian_scale);
build_extern_accuracy!(scale, opendp_accuracy__gaussian_scale_to_accuracy, gaussian_scale_to_accuracy);
build_extern_accuracy!(accuracy, opendp_accuracy__accuracy_to_gaussian_scale, accuracy_to_gaussian_scale);
