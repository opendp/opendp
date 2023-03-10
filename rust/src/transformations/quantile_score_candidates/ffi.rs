use std::ffi::{c_char, c_uint};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::Type,
    },
    traits::{Float, Number},
    transformations::{make_quantile_score_candidates, make_sized_quantile_score_candidates},
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_quantile_score_candidates(
    candidates: *const AnyObject,
    alpha: *const AnyObject,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TOA>(
        candidates: *const AnyObject,
        alpha: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: 'static + Number,
        TOA: 'static + Float,
    {
        let candidates = try_!(try_as_ref!(candidates).downcast_ref::<Vec<TIA>>()).clone();
        let alpha = *try_!(try_as_ref!(alpha).downcast_ref::<TOA>());
        make_quantile_score_candidates::<TIA, TOA>(candidates, alpha).into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (TIA, @numbers),
        (TOA, @floats)
    ], (candidates, alpha))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_quantile_score_candidates(
    size: c_uint,
    candidates: *const AnyObject,
    alpha: *const AnyObject,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TOA>(
        size: usize,
        candidates: *const AnyObject,
        alpha: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: 'static + Number,
        TOA: 'static + Float,
    {
        let candidates = try_!(try_as_ref!(candidates).downcast_ref::<Vec<TIA>>()).clone();
        let alpha = *try_!(try_as_ref!(alpha).downcast_ref::<TOA>());
        make_sized_quantile_score_candidates::<TIA, TOA>(size, candidates, alpha).into_any()
    }
    let size = size as usize;
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (TIA, @numbers),
        (TOA, @floats)
    ], (size, candidates, alpha))
}
