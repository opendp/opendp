use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};

use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::InfCast;
use crate::trans::{
    make_bounded_float_checked_sum, make_sized_bounded_float_checked_sum, Float, Pairwise,
    Sequential, UncheckedSum,
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_float_checked_sum(
    size_limit: c_uint,
    bounds: *const AnyObject,
    S: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        S: Type,
        size_limit: usize,
        bounds: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + Float,
    {
        fn monomorphize2<S>(
            size_limit: usize,
            bounds: (S::Item, S::Item),
        ) -> FfiResult<*mut AnyTransformation>
        where
            S: UncheckedSum,
            S::Item: 'static + Float,
        {
            make_bounded_float_checked_sum::<S>(size_limit, bounds).into_any()
        }
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        dispatch!(monomorphize2, [(S, [Sequential<T>, Pairwise<T>])], (size_limit, bounds))
    }
    let size_limit = size_limit as usize;
    let S = try_!(Type::try_from(S));
    let T = try_!(S.get_atom());
    dispatch!(monomorphize, [(T, @floats)], (S, size_limit, bounds))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_float_checked_sum(
    size: c_uint,
    bounds: *const AnyObject,
    S: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        S: Type,
        size: usize,
        bounds: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + Float,
        IntDistance: InfCast<T>,
    {
        fn monomorphize2<S>(
            size: usize,
            bounds: (S::Item, S::Item),
        ) -> FfiResult<*mut AnyTransformation>
        where
            S: UncheckedSum,
            S::Item: 'static + Float,
        {
            make_sized_bounded_float_checked_sum::<S>(size, bounds).into_any()
        }
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        dispatch!(monomorphize2, [(S, [Sequential<T>, Pairwise<T>])], (size, bounds))
    }
    let size = size as usize;
    let S = try_!(Type::try_from(S));
    let T = try_!(S.get_atom());
    dispatch!(monomorphize, [(T, @floats)], (S, size, bounds))
}
