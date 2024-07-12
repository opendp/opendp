use std::{ffi::c_char, sync::Arc};

use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyObject, AnyQueryable, Downcast},
        util::to_str,
    },
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery, OPENDP_LIB_PATH},
};

#[bootstrap(
    name = "set_opendp_lib_path",
    arguments(opendp_lib_path(c_type = "char *", rust_type = b"null")),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Configure the path to the OpenDP Library binary.
///
/// # Arguments
/// * `opendp_lib_path` - Absolute path to the OpenDP Library binary.
#[no_mangle]
pub extern "C" fn opendp_data__set_opendp_lib_path(
    opendp_lib_path: *mut c_char,
) -> FfiResult<*mut ()> {
    let opendp_lib_path = Arc::from(try_!(to_str(opendp_lib_path)).to_string());
    try_!(OPENDP_LIB_PATH
        .try_lock()
        .map_err(|_| err!(FFI, "failed to set OPENDP_LIB_PATH due to lock poisoning")))
    .replace(opendp_lib_path);
    Ok(()).into()
}

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
/// # Features
/// * `honest-but-curious` - LazyFrames can be collected an unlimited number of times.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[no_mangle]
pub extern "C" fn opendp_data__onceframe_lazy(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let answer: LazyFrame = try_!(queryable.eval_internal(&ExtractLazyFrame));
    Ok(AnyObject::new(answer)).into()
}
