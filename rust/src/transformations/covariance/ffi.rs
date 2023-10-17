use std::{
    convert::TryFrom,
    os::raw::{c_char, c_uint},
};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::Type,
    },
    traits::{CheckAtom, Float},
    transformations::{make_sized_bounded_covariance, Pairwise, Sequential, UncheckedSum}, error::Fallible,
};

// no entry in bootstrap.json because there's no way to get data into it
#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_bounded_covariance(
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
    ) -> Fallible<AnyTransformation>
    where
        T: 'static + Float,
        (T, T): CheckAtom,
    {
        fn monomorphize2<S>(
            size: usize,
            bounds_0: (S::Item, S::Item),
            bounds_1: (S::Item, S::Item),
            ddof: usize,
        ) -> Fallible<AnyTransformation>
        where
            S: UncheckedSum,
            S::Item: 'static + Float,
            (S::Item, S::Item): CheckAtom,
        {
            make_sized_bounded_covariance::<S>(size, bounds_0, bounds_1, ddof).into_any()
        }
        let bounds_0 = *try_as_ref!(bounds_0).downcast_ref::<(T, T)>()?;
        let bounds_1 = *try_as_ref!(bounds_1).downcast_ref::<(T, T)>()?;
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
    ], (size, bounds_0, bounds_1, ddof, S)).into()
}
