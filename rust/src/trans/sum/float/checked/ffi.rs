use std::convert::TryFrom;
use std::iter::Sum;
use std::os::raw::{c_char, c_uint};

use num::One;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};

use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::samplers::Shuffle;
use crate::traits::{CheckNull, DistanceConstant, ExactIntCast, InfCast, InfSub, InfAdd, InfPow, FloatBits, AlertingAbs};
use crate::trans::make_sized_bounded_float_checked_sum;

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_float_checked_sum(
    size: c_uint,
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(size: usize, bounds: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        T: DistanceConstant<IntDistance>
            + ExactIntCast<T::Bits>
            + InfAdd
            + InfPow
            + One
            + FloatBits
            + ExactIntCast<usize>
            + InfSub
            + AlertingAbs
            + CheckNull,
        for<'a> T: Sum<&'a T>,
        Vec<T>: Shuffle,
        IntDistance: InfCast<T>,
    {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_float_checked_sum::<T>(size, bounds).into_any()
    }
    let size = size as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @floats)], (size, bounds))
}
