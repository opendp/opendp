use crate::{core::FfiResult, error::Fallible};

use super::{any::AnyObject, util::into_owned};

pub type CallbackFn = extern "C" fn(*const AnyObject) -> *mut FfiResult<*mut AnyObject>;

// wrap a CallbackFn in a closure, so that it can be used in transformations and measurements
pub fn wrap_func(func: CallbackFn) -> impl Fn(&AnyObject) -> Fallible<AnyObject> {
    move |arg: &AnyObject| -> Fallible<AnyObject> {
        into_owned(func(arg as *const AnyObject))?.into()
    }
}
