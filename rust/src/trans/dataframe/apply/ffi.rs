use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::err;
use crate::trans::{make_df_cast_default, make_df_is_equal};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{Hashable, Primitive, RoundCast};

#[no_mangle]
pub extern "C" fn opendp_trans__make_df_cast_default(
    column_name: *const AnyObject,
    TK: *const c_char,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK, TIA, TOA>(
        column_name: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TK: Hashable,
        TIA: Primitive,
        TOA: Primitive + RoundCast<TIA>,
    {
        let column_name: TK = try_!(try_as_ref!(column_name).downcast_ref::<TK>()).clone();
        make_df_cast_default::<TK, TIA, TOA>(column_name).into_any()
    }
    let TK = try_!(Type::try_from(TK));
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TIA, @primitives),
        (TOA, @primitives)
    ], (column_name))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_df_is_equal(
    column_name: *const AnyObject,
    value: *const AnyObject,
    TK: *const c_char,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK, TIA>(
        column_name: *const AnyObject,
        value: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TK: Hashable,
        TIA: Primitive,
    {
        let column_name: TK = try_!(try_as_ref!(column_name).downcast_ref::<TK>()).clone();
        let value: TIA = try_!(try_as_ref!(value).downcast_ref::<TIA>()).clone();
        make_df_is_equal::<TK, TIA>(column_name, value).into_any()
    }
    let TK = try_!(Type::try_from(TK));
    let TIA = try_!(Type::try_from(TIA));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TIA, @primitives)
    ], (column_name, value))
}

