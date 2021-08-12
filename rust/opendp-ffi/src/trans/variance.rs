use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::{Add, Div, Sub};
use std::os::raw::{c_char, c_uint, c_void};

use num::{Float, One, Zero};

use opendp::err;
use opendp::traits::{DistanceConstant, InfCast, ExactIntCast, CheckedMul, CheckNull};
use opendp::trans::{make_bounded_covariance, make_bounded_variance};

use crate::any::{AnyObject, AnyTransformation, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;
use opendp::dist::IntDistance;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_variance(
    lower: *const c_void, upper: *const c_void,
    length: c_uint, ddof: c_uint,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize2<T>(
        lower: *const c_void, upper: *const c_void, length: usize, ddof: usize,
    ) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant<IntDistance> + Float + for<'a> Sum<&'a T> + Sum<T> + ExactIntCast<usize> + CheckedMul + CheckNull,
              for<'a> &'a T: Sub<Output=T> + Add<&'a T, Output=T>,
              IntDistance: InfCast<T> {
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        make_bounded_variance::<T>(lower, upper, length, ddof).into_any()
    }

    let length = length as usize;
    let ddof = ddof as usize;
    let T = try_!(Type::try_from(T));

    dispatch!(monomorphize2, [
        (T, @floats)
    ], (lower, upper, length, ddof))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_covariance(
    lower: *const AnyObject, upper: *const AnyObject,
    length: c_uint, ddof: c_uint,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        lower: *const AnyObject,
        upper: *const AnyObject,
        length: usize, ddof: usize,
    ) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant<IntDistance> + Sub<Output=T> + Sum<T> + Zero + One + ExactIntCast<usize> + CheckedMul + CheckNull,
              for<'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
              for<'a> &'a T: Sub<Output=T>,
              IntDistance: InfCast<T> {
        let lower = try_!(try_as_ref!(lower).downcast_ref::<(T, T)>()).clone();
        let upper = try_!(try_as_ref!(upper).downcast_ref::<(T, T)>()).clone();
        make_bounded_covariance::<T>(lower, upper, length, ddof).into_any()
    }
    let length = length as usize;
    let ddof = ddof as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T, @floats)
    ], (lower, upper, length, ddof))
}
