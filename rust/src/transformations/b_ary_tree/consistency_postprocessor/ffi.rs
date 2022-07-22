
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
    branching_factor: c_uint,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TOA>(
        branching_factor: usize
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: 'static + CheckNull + Clone,
        TOA: Float + RoundCast<TIA>,
    {
        make_b_ary_tree_consistent::<TIA, TOA>(branching_factor).into_any()
    }

    let branching_factor = branching_factor as usize;
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (TIA, @integers),
        (TOA, @floats)
    ], (branching_factor))
}
