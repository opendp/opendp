use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{CheckNull, Hashable, Primitive, TotalOrd};
use crate::trans::make_select_array;

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_array(
    col_names: *const AnyObject,
    K: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K: Hashable, TOA: Primitive>(
        col_names: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        K: 'static + Clone + TotalOrd + CheckNull,
    {
        let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<K>>()).clone();
        make_select_array::<K, TOA>(col_names).into_any()
    }
    let K = try_!(Type::try_from(K));
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (K, @hashable),
        (TOA, @primitives)
    ], (col_names))
}
