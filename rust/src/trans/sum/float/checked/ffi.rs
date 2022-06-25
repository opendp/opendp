use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};

use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::InfCast;
use crate::trans::{make_sized_bounded_float_checked_sum, Float, Pairwise, make_bounded_float_checked_sum};

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_float_checked_sum(
    size_limit: c_uint,
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(size_limit: usize, bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + Float,
        IntDistance: InfCast<T>,
    {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_bounded_float_checked_sum::<Pairwise<T>>(size_limit, bounds).into_any()
    }
    let size_limit = size_limit as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @floats)], (size_limit, bounds))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_float_checked_sum(
    size: c_uint,
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(size: usize, bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + Float,
        IntDistance: InfCast<T>,
    {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_float_checked_sum::<Pairwise<T>>(size, bounds).into_any()
    }
    let size = size as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @floats)], (size, bounds))
}
