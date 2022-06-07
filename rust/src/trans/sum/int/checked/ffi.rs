use std::convert::TryFrom;
use std::iter::Sum;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};

use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{DistanceConstant, ExactIntCast, InfSub, CheckNull, InfDiv, InfCast};
use crate::trans::sum::int::AddIsExact;
use crate::trans::{make_sized_bounded_int_checked_sum};

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_int_checked_sum(
    size: c_uint, 
    bounds: *const AnyObject,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        size: usize, bounds: *const AnyObject
    ) -> FfiResult<*mut AnyTransformation>
        where T: 'static + DistanceConstant<IntDistance>
            + ExactIntCast<usize>
            + InfSub
            + CheckNull
            + InfDiv
            + AddIsExact,
        for<'a> T: Sum<&'a T>,
        IntDistance: InfCast<T>, {
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        make_sized_bounded_int_checked_sum::<T>(size, bounds).into_any()
    }
    let size = size as usize;
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @integers)], (size, bounds))
}