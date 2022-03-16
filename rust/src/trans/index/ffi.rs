use std::convert::TryFrom;
use std::hash::Hash;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation};
use crate::ffi::any::Downcast;
use crate::ffi::util::Type;
use crate::traits::CheckNull;
use crate::trans::{make_find, make_find_bin, make_index};

#[no_mangle]
pub extern "C" fn opendp_trans__make_find(
    categories: *const AnyObject,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA>(
        categories: *const AnyObject
    ) -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + CheckNull + Clone + Hash + Eq {
        let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<TIA>>()).clone();
        make_find::<TIA>(categories).into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    dispatch!(monomorphize, [
        (TIA, @hashable)
    ], (categories))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_find_bin(
    edges: *const AnyObject,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA>(
        edges: *const AnyObject
    ) -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + Clone + PartialOrd + CheckNull {
        let edges = try_!(try_as_ref!(edges).downcast_ref::<Vec<TIA>>()).clone();
        make_find_bin::<TIA>(edges).into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    dispatch!(monomorphize, [
        (TIA, @numbers)
    ], (edges))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_index(
    categories: *const AnyObject,
    null: *const AnyObject,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TOA>(
        edges: *const AnyObject,
        null: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
        where TOA: 'static + Clone + PartialOrd + CheckNull {
        let edges = try_!(try_as_ref!(edges).downcast_ref::<Vec<TOA>>()).clone();
        let null = try_!(try_as_ref!(null).downcast_ref::<TOA>()).clone();
        make_index::<TOA>(edges, null).into_any()
    }
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (TOA, @primitives)
    ], (categories, null))
}
