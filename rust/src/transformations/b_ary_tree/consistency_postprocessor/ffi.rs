use std::{convert::TryFrom, os::raw::c_char};

use crate::{
    core::{FfiResult, IntoAnyFunctionFfiResultExt},
    error::Fallible,
    ffi::{any::AnyFunction, util::Type},
    traits::{CheckAtom, Float, RoundCast},
    transformations::make_consistent_b_ary_tree,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_consistent_b_ary_tree(
    branching_factor: u32,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyFunction> {
    fn monomorphize<TIA, TOA>(branching_factor: u32) -> Fallible<AnyFunction>
    where
        TIA: 'static + CheckAtom + Clone,
        TOA: Float + RoundCast<TIA>,
    {
        make_consistent_b_ary_tree::<TIA, TOA>(branching_factor).into_any()
    }

    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (TIA, @integers),
        (TOA, @floats)
    ], (branching_factor))
    .into()
}
