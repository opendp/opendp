use std::{os::raw::{c_char, c_void}, convert::TryFrom};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{any::{AnyObject, AnyTransformation, Downcast}, util::Type},
    traits::Float,
    trans::{make_postprocess_sized_proportion_ci, make_postprocess_proportion_ci},
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_postprocess_sized_proportion_ci(
    strat_sizes: *const AnyObject,
    sample_sizes: *const AnyObject,
    scale: *const c_void,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA: Float>(
        strat_sizes: Vec<usize>,
        sample_sizes: Vec<usize>,
        scale: *const c_void,
    ) -> FfiResult<*mut AnyTransformation> {
        let scale = *try_as_ref!(scale as *const TA);
        make_postprocess_sized_proportion_ci::<TA>(strat_sizes, sample_sizes, scale).into_any()
    }
    let strat_sizes = try_!(try_as_ref!(strat_sizes).downcast_ref::<Vec<usize>>()).clone();
    let sample_sizes = try_!(try_as_ref!(sample_sizes).downcast_ref::<Vec<usize>>()).clone();
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [
        (TA, @floats)
    ], (strat_sizes, sample_sizes, scale))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_postprocess_proportion_ci(
    strat_sizes: *const AnyObject,
    sum_scale: *const c_void,
    size_scale: *const c_void,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA: Float>(
        strat_sizes: Vec<usize>,
        sum_scale: *const c_void,
        size_scale: *const c_void,
    ) -> FfiResult<*mut AnyTransformation> {
        let sum_scale = *try_as_ref!(sum_scale as *const TA);
        let size_scale = *try_as_ref!(size_scale as *const TA);
        make_postprocess_proportion_ci::<TA>(strat_sizes, sum_scale, size_scale).into_any()
    }
    let strat_sizes = try_!(try_as_ref!(strat_sizes).downcast_ref::<Vec<usize>>()).clone();
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [
        (TA, @floats)
    ], (strat_sizes, sum_scale, size_scale))
}
