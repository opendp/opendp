use opendp_derive::bootstrap;

pub(super) mod preflight;
use polars::lazy::frame::LazyFrame;

use crate::{
    core::{FfiResult, FfiSlice},
    ffi::any::{AnyObject, AnyQueryable, Downcast},
    ffi::util::{self, c_bool},
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery},
};

#[bootstrap(
    name = "validate_polars_plan",
    arguments(serialized_plan(rust_type = "Vec<u8>", hint = "Sequence[int]")),
    returns(c_type = "FfiResult<bool *>", hint = "bool")
)]
/// Reject unsafe serialized Polars plans before invoking Polars' deserializer.
///
/// This does not execute any user-supplied Python objects. Python callers must
/// invoke this before `pl.LazyFrame.deserialize` on externally supplied bytes.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__validate_polars_plan(
    serialized_plan: *const FfiSlice,
) -> FfiResult<*mut c_bool> {
    let serialized_plan = try_as_ref!(serialized_plan);
    if serialized_plan.ptr.is_null() || serialized_plan.len > isize::MAX as usize {
        return err!(FFI, "Invalid serialized Polars plan buffer").into();
    }
    // SAFETY: FFI callers provide an in-bounds byte buffer that lives for the
    // call; the generated Python Vec<u8> converter owns the input allocation.
    let bytes = unsafe {
        std::slice::from_raw_parts(
            serialized_plan.ptr as *const u8,
            serialized_plan.len,
        )
    };
    try_!(preflight::validate_serialized_dslplan(bytes));
    FfiResult::Ok(util::into_raw(util::from_bool(true)))
}

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
