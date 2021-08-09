use crate::any::{AnyQueryable, AnyObject};
use crate::core::FfiResult;
use std::os::raw::c_char;
use opendp::err;
use crate::util::into_c_char_p;
use crate::util;

#[no_mangle]
pub extern "C" fn opendp_core__queryable_query_type(this: *mut AnyQueryable) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    let state = try_!(this.state.as_ref().ok_or_else(|| err!(FFI, "cannot retrieve type of None state")));
    FfiResult::Ok(try_!(into_c_char_p(state.type_q.descriptor.to_string())))
}

#[no_mangle]
pub extern "C" fn opendp_core__queryable_invoke(
    queryable: *mut AnyQueryable, query: *const AnyObject
) -> FfiResult<*mut AnyObject> {
    let queryable = try_as_mut_ref!(queryable);
    let query = try_as_ref!(query);
    queryable.eval(query).into()
}

#[no_mangle]
pub extern "C" fn opendp_core___queryable_free(this: *mut AnyQueryable) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

