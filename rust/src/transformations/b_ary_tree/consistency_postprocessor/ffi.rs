
use std::{
    convert::TryFrom,
    os::raw::{c_char, c_uint},
};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{any::AnyTransformation, util::Type},
    traits::{CheckNull, Float, RoundCast},
    trans::make_b_ary_tree_consistent
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_b_ary_tree_consistent(
    b: c_uint,
    TI: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TI, TO>(
        b: usize
    ) -> FfiResult<*mut AnyTransformation>
    where
        TI: 'static + CheckNull + Clone,
        TO: Float + RoundCast<TI>,
    {
        make_b_ary_tree_consistent::<TI, TO>(b).into_any()
    }

    let b = b as usize;
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [
        (TI, @integers),
        (TO, @floats)
    ], (b))
}
