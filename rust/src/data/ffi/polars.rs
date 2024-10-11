use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;

use crate::{
    core::FfiResult,
    ffi::any::{AnyObject, AnyQueryable, Downcast},
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery},
};

#[bootstrap(
    name = "onceframe_collect",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Collects a DataFrame from a OnceFrame, exhausting the OnceFrame.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[no_mangle]
pub extern "C" fn opendp_data__onceframe_collect(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let query = AnyObject::new(OnceFrameQuery::Collect);
    let answer: OnceFrameAnswer = try_!(try_!(queryable.eval(&query)).downcast());
    let OnceFrameAnswer::Collect(frame) = answer;

    Ok(AnyObject::new(frame)).into()
}

#[bootstrap(
    features("honest-but-curious"),
    name = "onceframe_lazy",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Extracts a LazyFrame from a OnceFrame,
/// circumventing protections against multiple evaluations.
///
/// Each collection consumes the entire allocated privacy budget.
/// To remain DP at the advertised privacy level, only collect the LazyFrame once.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
///
/// # Why honest-but-curious?
/// The privacy guarantees only apply if:
/// 1. The LazyFrame (compute plan) is only ever executed once.
/// 2. The analyst does not observe ordering of rows in the output. To ensure this, shuffle the output.
#[no_mangle]
pub extern "C" fn opendp_data__onceframe_lazy(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let answer: LazyFrame = try_!(queryable.eval_internal(&ExtractLazyFrame));
    Ok(AnyObject::new(answer)).into()
}
