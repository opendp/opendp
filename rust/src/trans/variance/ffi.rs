use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::AbsoluteDistance;
use crate::dom::AllDomain;
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::trans::{make_sized_bounded_variance, UncheckedSum, Sequential, Pairwise, LipschitzMulDomain, LipschitzMulMetric, Float};


#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_variance(
    size: c_uint,
    bounds: *const AnyObject,
    ddof: c_uint,
    S: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        size: usize,
        bounds: *const AnyObject,
        ddof: usize,
        S: Type,
    ) -> FfiResult<*mut AnyTransformation>
    where
        T: 'static + Float,
    {
        fn monomorphize2<S>(
            size: usize,
            bounds: (S::Item, S::Item),
            ddof: usize
        ) -> FfiResult<*mut AnyTransformation>
        where
            S: UncheckedSum,
            S::Item: 'static + Float,
            AllDomain<S::Item>: LipschitzMulDomain<Atom = S::Item>,
            AbsoluteDistance<S::Item>: LipschitzMulMetric<Distance = S::Item>,
        {
            make_sized_bounded_variance::<S>(size, bounds, ddof).into_any()
        }
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T, T)>()).clone();
        dispatch!(monomorphize2, [(S, [Sequential<T>, Pairwise<T>])], (size, bounds, ddof))
    }
    let size = size as usize;
    let ddof = ddof as usize;
    let S = try_!(Type::try_from(S));
    let T = try_!(S.get_atom());
    dispatch!(monomorphize, [
        (T, @floats)
    ], (size, bounds, ddof, S))
}
