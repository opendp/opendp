use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::{Add, Sub};
use std::os::raw::{c_char, c_uint};

use num::Float;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{
    AlertingAbs, CheckNull, DistanceConstant, ExactIntCast, FloatBits, InfAdd, InfCast, InfDiv,
    InfPow, InfSub,
};
use crate::trans::make_sized_bounded_sum_of_squared_deviations;

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_sum_of_squared_deviations(
    size: c_uint,
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize2<T>(size: usize, bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        T: DistanceConstant<IntDistance>
            + Float
            + for<'a> Sum<&'a T>
            + Sum<T>
            + ExactIntCast<usize>
            + InfDiv
            + InfSub
            + InfAdd
            + CheckNull
            + InfPow
            + FloatBits
            + ExactIntCast<T::Bits>
            + AlertingAbs,
        for<'a> &'a T: Sub<Output = T> + Add<&'a T, Output = T>,
        IntDistance: InfCast<T>,
    {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_sum_of_squared_deviations::<T>(size, bounds).into_any()
    }

    let size = size as usize;
    let T = try_!(Type::try_from(T));

    dispatch!(monomorphize2, [
        (T, @floats)
    ], (size, bounds))
}
