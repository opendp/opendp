use std::{ffi::c_char, path::PathBuf};

use opendp_derive::bootstrap;
use polars::{
    lazy::frame::LazyFrame,
    prelude::{CsvWriterOptions, ParquetWriteOptions},
};

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyObject, AnyQueryable, Downcast},
        util::to_str,
    },
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
    let OnceFrameAnswer::Collect(frame) = answer else {
        return err!(FFI, "Expected Collect answer.").into();
    };

    Ok(AnyObject::new(frame)).into()
}

#[bootstrap(
    name = "onceframe_sink_csv",
    arguments(
        onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"),
        path(c_type = "const char *", rust_type = "String")
    )
)]
/// Internal function. Sinks the data from a OnceFrame into a CSV file, exhausting the OnceFrame.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[no_mangle]
pub extern "C" fn opendp_data__onceframe_sink_csv(
    onceframe: *mut AnyObject,
    path: *const c_char,
) -> FfiResult<*mut ()> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let path = PathBuf::from(try_!(to_str(path)));

    // TODO: allow options to be passed in
    let options = CsvWriterOptions::default();

    let query = AnyObject::new(OnceFrameQuery::SinkCsv(path, options));

    try_!(queryable.eval(&query));
    Ok(()).into()
}

#[bootstrap(
    name = "onceframe_sink_parquet",
    arguments(
        onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"),
        path(c_type = "const char *", rust_type = "String")
    )
)]
/// Internal function. Sinks the data from a OnceFrame into a Parquet file, exhausting the OnceFrame.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[no_mangle]
pub extern "C" fn opendp_data__onceframe_sink_parquet(
    onceframe: *mut AnyObject,
    path: *const c_char,
) -> FfiResult<*mut ()> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let path = PathBuf::from(try_!(to_str(path)));

    // TODO: allow options to be passed in
    let options = ParquetWriteOptions::default();

    let query = AnyObject::new(OnceFrameQuery::SinkParquet(path, options));

    try_!(queryable.eval(&query));
    Ok(()).into()
}

#[bootstrap(
    name = "_onceframe_extract_lazyframe",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Extracts a LazyFrame from a OnceFrame,
/// circumventing protections against multiple evaluations.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[no_mangle]
pub extern "C" fn opendp_data___onceframe_extract_lazyframe(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let answer: LazyFrame = try_!(queryable.eval_internal(&ExtractLazyFrame));
    Ok(AnyObject::new(answer)).into()
}
