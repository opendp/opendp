use std::ffi::c_char;

use opendp_derive::bootstrap;

pub(super) mod preflight;
use polars::lazy::frame::LazyFrame;

use crate::{
    core::{FfiResult, FfiSlice},
    domains::BitVector,
    ffi::any::{AnyObject, AnyQueryable, Downcast},
    ffi::util,
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery},
};

#[bootstrap(
    name = "sanitize_polars_plan",
    arguments(
        serialized_plan(rust_type = "BitVector", hint = "bytes"),
        trusted_lib(c_type = "char *", rust_type = "String")
    ),
    returns(
        c_type = "FfiResult<AnyObject *>",
        rust_type = "BitVector",
        hint = "bytes"
    )
)]
/// Validate a serialized plan and rebind approved OpenDP shims to a server-owned library.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__sanitize_polars_plan(
    serialized_plan: *const FfiSlice,
    trusted_lib: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let serialized_plan = try_as_ref!(serialized_plan);
    let trusted_lib = try_!(util::to_str(trusted_lib));
    if serialized_plan.ptr.is_null() || serialized_plan.len % 8 != 0 {
        return err!(FFI, "Invalid serialized Polars plan buffer").into();
    }
    let len = serialized_plan.len / 8;
    if len > preflight::MAX_PLAN_BYTES {
        return err!(FFI, "Serialized Polars plan exceeds the size limit").into();
    }
    let bytes = unsafe { std::slice::from_raw_parts(serialized_plan.ptr as *const u8, len) };
    let sanitized = try_!(preflight::sanitize_serialized_dslplan(bytes, trusted_lib));
    Ok(AnyObject::new(BitVector::from_vec(sanitized))).into()
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
