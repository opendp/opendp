use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::err;
use crate::trans::make_filter_by;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::Hashable;

#[no_mangle]
pub extern "C" fn opendp_trans__make_filter_by(
    identifier_column: *const AnyObject,
    keep_columns: *const AnyObject,
    TK: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK>(
        identifier_column: *const AnyObject,
        keep_columns: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TK: Hashable,
    {
        let identifier_column: TK =
            try_!(try_as_ref!(identifier_column).downcast_ref::<TK>()).clone();
        let keep_columns: Vec<TK> = try_!(try_as_ref!(keep_columns).downcast_ref::<Vec<TK>>()).clone();
        make_filter_by::<TK>(identifier_column, keep_columns).into_any()
    }
    let TK = try_!(Type::try_from(TK));

    dispatch!(monomorphize, [
        (TK, @hashable)
    ], (identifier_column, keep_columns))
}
