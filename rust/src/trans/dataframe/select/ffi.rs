use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::err;
use crate::trans::make_select_column;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{Hashable, Primitive};

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(
    key: *const AnyObject,
    K: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K, TOA>(key: *const AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        K: Hashable,
        TOA: Primitive,
    {
        let key: K = try_!(try_as_ref!(key).downcast_ref::<K>()).clone();
        make_select_column::<K, TOA>(key).into_any()
    }
    let K = try_!(Type::try_from(K));
    let TOA = try_!(Type::try_from(TOA));

    dispatch!(monomorphize, [
        (K, @hashable),
        (TOA, @primitives)
    ], (key))
}
