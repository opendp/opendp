use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::error::Fallible;
#[allow(deprecated)]
use crate::transformations::make_select_column;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{Hashable, Primitive};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_select_column(
    key: *const AnyObject,
    K: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K, TOA>(key: *const AnyObject) -> Fallible<AnyTransformation>
    where
        K: Hashable,
        TOA: Primitive,
    {
        let key: K = try_as_ref!(key).downcast_ref::<K>()?.clone();
        #[allow(deprecated)]
        make_select_column::<K, TOA>(key).into_any()
    }
    let K = try_!(Type::try_from(K));
    let TOA = try_!(Type::try_from(TOA));

    dispatch!(monomorphize, [
        (K, @hashable),
        (TOA, @primitives)
    ], (key))
    .into()
}
