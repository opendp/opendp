use std::{os::raw::c_char, path::Path};

use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyObject, AnyQueryable, Downcast},
        util,
    },
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery, TakeLazyFrame, split::split_plan},
};

#[bootstrap(
    name = "onceframe_collect",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Collects a DataFrame from a OnceFrame, exhausting the OnceFrame.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[unsafe(no_mangle)]
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
///
/// 1. The LazyFrame (compute plan) is only ever executed once.
/// 2. The analyst does not observe ordering of rows in the output.
///
/// To ensure that row ordering is not observed:
///
/// 1. Do not extend the compute plan with order-sensitive computations.
/// 2. Shuffle the output once collected ([in Polars sample all, with shuffling enabled](https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.sample.html)).
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_lazy(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let answer: LazyFrame = try_!(queryable.eval_internal(&ExtractLazyFrame));
    Ok(AnyObject::new(answer)).into()
}

#[bootstrap(
    features("contrib", "honest-but-curious"),
    name = "onceframe_split",
    arguments(
        onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"),
        work_dir(rust_type = "String")
    )
)]
/// Internal executor-only function. Consumes a OnceFrame and splits its native plan.
///
/// The OnceFrame is exhausted before splitting, even if splitting fails.
/// Returns a two-element list of native LazyFrames, in this order:
/// `[SinkMultiple, continuation]`. Splitting performs no execution, I/O or schema resolution.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
/// * `work_dir` - Protected, executor-owned directory for intermediate IPC files.
///
/// # Why honest-but-curious?
/// Both plans and the intermediate files expose deterministic private information.
/// They are internal executor artifacts, not differentially private releases, and
/// must never be exposed to the analyst. The trusted executor must execute each
/// sink input exactly once against a consistent snapshot, materialize its complete
/// tabular result at the native sink path, then execute the continuation once.
/// It must protect and clean up all intermediate files on success or failure,
/// preserve native plugins and plan semantics, and shuffle the complete final
/// output before releasing it without exposing row order. Do not retry execution
/// after consumption or extend the plans with analyst-controlled computations.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_split(
    onceframe: *mut AnyObject,
    work_dir: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());
    let work_dir = Path::new(try_!(util::to_str(work_dir)));

    let lazyframe: LazyFrame = try_!(queryable.eval_internal(&TakeLazyFrame));
    let (sinks, continuation) = try_!(split_plan(lazyframe.logical_plan, work_dir));
    Ok(AnyObject::new(vec![
        AnyObject::new(LazyFrame::from(sinks)),
        AnyObject::new(LazyFrame::from(continuation)),
    ]))
    .into()
}
