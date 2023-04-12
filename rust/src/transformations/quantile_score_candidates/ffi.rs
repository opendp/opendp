use std::ffi::{c_char, c_uint};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::Type,
    },
    traits::Number,
    transformations::{
        make_quantile_score_candidates, make_sized_quantile_score_candidates,
        quantile_score_candidates::IntoFrac,
    },
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_quantile_score_candidates(
    candidates: *const AnyObject,
    alpha: *const AnyObject,
    TIA: *const c_char,
    F: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, F>(
        candidates: *const AnyObject,
        alpha: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: 'static + Number,
        F: 'static + IntoFrac + Clone,
    {
        let candidates = try_!(try_as_ref!(candidates).downcast_ref::<Vec<TIA>>()).clone();
        let alpha = try_!(try_as_ref!(alpha).downcast_ref::<F>()).clone();
        make_quantile_score_candidates::<TIA, F>(candidates, alpha).into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let F = try_!(Type::try_from(F));
    dispatch!(monomorphize, [
        (TIA, @numbers),
        (F, [f32, f64, (usize, usize), (i32, i32)])
    ], (candidates, alpha))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_quantile_score_candidates(
    size: c_uint,
    candidates: *const AnyObject,
    alpha: *const AnyObject,
    TIA: *const c_char,
    F: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, F>(
        size: usize,
        candidates: *const AnyObject,
        alpha: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: 'static + Number,
        F: 'static + IntoFrac + Clone,
    {
        let candidates = try_!(try_as_ref!(candidates).downcast_ref::<Vec<TIA>>()).clone();
        let alpha = try_!(try_as_ref!(alpha).downcast_ref::<F>()).clone();
        make_sized_quantile_score_candidates::<TIA, F>(size, candidates, alpha).into_any()
    }
    let size = size as usize;
    let TIA = try_!(Type::try_from(TIA));
    let F = try_!(Type::try_from(F));
    dispatch!(monomorphize, [
        (TIA, @numbers),
        (F, [f32, f64, (usize, usize), (i32, i32)])
    ], (size, candidates, alpha))
}
