use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};
use std::os::raw::{c_char, c_uint};

use num::{Float, Zero};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{CheckNull, DistanceConstant, ExactIntCast, InfAdd, InfCast, InfSub};
use crate::trans::{make_sized_bounded_covariance, make_sized_bounded_variance};

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_variance(
    size: c_uint, bounds: *const AnyObject,
    ddof: c_uint,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize2<T>(
        size: usize, bounds: *const AnyObject, ddof: usize,
    ) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant<IntDistance> + Float + for<'a> Sum<&'a T> + Sum<T> + ExactIntCast<usize> + InfSub + InfAdd + CheckNull,
              for<'a> &'a T: Sub<Output=T> + Add<&'a T, Output=T>,
              IntDistance: InfCast<T> {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_variance::<T>(size, bounds, ddof).into_any()
    }

    let size = size as usize;
    let ddof = ddof as usize;
    let T = try_!(Type::try_from(T));

    dispatch!(monomorphize2, [
        (T, @floats)
    ], (size, bounds, ddof))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_covariance(
    size: c_uint, lower: *const AnyObject, upper: *const AnyObject,
    ddof: c_uint, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        size: usize,
        lower: *const AnyObject, upper: *const AnyObject,
        ddof: usize,
    ) -> FfiResult<*mut AnyTransformation> where
        T: ExactIntCast<usize> + DistanceConstant<IntDistance> + Zero
        + Add<Output=T> + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T>
        + InfAdd + InfSub + CheckNull,
        for<'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
        for<'a> &'a T: Sub<Output=T>,
        IntDistance: InfCast<T> {
        let lower = try_!(try_as_ref!(lower).downcast_ref::<(T, T)>()).clone();
        let upper = try_!(try_as_ref!(upper).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_covariance::<T>(size, lower, upper, ddof).into_any()
    }
    let size = size as usize;
    let ddof = ddof as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T, @floats)
    ], (size, lower, upper, ddof))
}
