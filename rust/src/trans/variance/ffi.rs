use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};
use std::os::raw::{c_char, c_uint};

use num::{Float, One, Zero};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{
    AlertingAbs, CheckNull, DistanceConstant, ExactIntCast, FloatBits, InfAdd, InfCast, InfDiv,
    InfMul, InfPow, InfSub, SaturatingMul,
};
use crate::trans::{make_sized_bounded_covariance, make_sized_bounded_variance};

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_variance(
    size: c_uint,
    bounds: *const AnyObject,
    ddof: c_uint,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize2<T>(
        size: usize,
        bounds: *const AnyObject,
        ddof: usize,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: DistanceConstant<IntDistance>
            + Float
            + One
            + Sum<T>
            + ExactIntCast<usize>
            + ExactIntCast<T::Bits>
            + InfMul
            + InfSub
            + InfAdd
            + InfDiv
            + CheckNull
            + InfPow
            + FloatBits
            + for<'a> Sum<&'a T>
            + AlertingAbs
            + for<'a> Mul<&'a T, Output = T>
            + InfCast<T>
            + SaturatingMul,
        for<'a> &'a T: Sub<Output = T> + Add<&'a T, Output = T>,
    {
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
    size: c_uint,
    bounds_0: *const AnyObject,
    bounds_1: *const AnyObject,
    ddof: c_uint,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        size: usize,
        bounds_0: *const AnyObject,
        bounds_1: *const AnyObject,
        ddof: usize,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: ExactIntCast<usize>
            + CheckNull
            + DistanceConstant<IntDistance>
            + ExactIntCast<T::Bits>
            + Sum<T>
            + Zero
            + Float
            + InfAdd
            + InfSub
            + InfDiv
            + InfPow
            + FloatBits
            + AlertingAbs
            + SaturatingMul,
        for<'a> T: Div<&'a T, Output = T> + Add<&'a T, Output = T> + Mul<&'a T, Output = T>,
        for<'a> &'a T: Sub<Output = T>,
    {
        let bounds_0 = try_!(try_as_ref!(bounds_0).downcast_ref::<(T, T)>()).clone();
        let bounds_1 = try_!(try_as_ref!(bounds_1).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_covariance::<T>(size, bounds_0, bounds_1, ddof).into_any()
    }
    let size = size as usize;
    let ddof = ddof as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T, @floats)
    ], (size, bounds_0, bounds_1, ddof))
}
