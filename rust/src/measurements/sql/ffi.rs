use std::{collections::HashMap, ffi::c_char};

use polars::prelude::LazyFrame;

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyObject, Downcast},
        util::to_str,
    },
    measurements::sql::sql_to_plan,
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__sql_to_plan(
    query: *const c_char,
    tables: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let query = try_!(to_str(query)).to_string();
    let tables = try_!(try_as_ref!(tables).downcast_ref::<HashMap<String, LazyFrame>>()).clone();

    sql_to_plan(query, tables).map(AnyObject::new).into()
}
