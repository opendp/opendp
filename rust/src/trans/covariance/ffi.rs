use std::{os::raw::{c_char, c_uint}, convert::TryFrom};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::Type,
    },
    trans::{make_sized_bounded_covariance, Pairwise, Sequential, UncheckedSum},
    traits::Float
};

// no entry in bootstrap.json because there's no way to get data into it
#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_covariance(
    size: c_uint,
    bounds_0: *const AnyObject,
    bounds_1: *const AnyObject,
    ddof: c_uint,
    S: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        size: usize,
        bounds_0: *const AnyObject,
        bounds_1: *const AnyObject,
        ddof: usize,
        S: Type,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + Float,
    {
        fn monomorphize2<S>(
            size: usize,
            bounds_0: (S::Item, S::Item),
            bounds_1: (S::Item, S::Item),
            ddof: usize,
        ) -> FfiResult<*mut AnyTransformation>
        where
            S: UncheckedSum,
            S::Item: 'static + Float,
        {
            make_sized_bounded_covariance::<S>(size, bounds_0, bounds_1, ddof).into_any()
        }
        let bounds_0 = try_!(try_as_ref!(bounds_0).downcast_ref::<(T, T)>()).clone();
        let bounds_1 = try_!(try_as_ref!(bounds_1).downcast_ref::<(T, T)>()).clone();
        dispatch!(monomorphize2, [
            (S, [Sequential<T>, Pairwise<T>])
        ], (size, bounds_0, bounds_1, ddof))
    }
    let size = size as usize;
    let ddof = ddof as usize;
    let S = try_!(Type::try_from(S));
    let T = try_!(S.get_atom());
    dispatch!(monomorphize, [
        (T, @floats)
    ], (size, bounds_0, bounds_1, ddof, S))
}
