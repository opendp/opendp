use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::error::Fallible;
use crate::ffi::any::{AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::{c_bool, to_bool, Type};
use crate::measurements::{make_randomized_response, make_randomized_response_bool};
use crate::traits::Hashable;

#[no_mangle]
pub extern "C" fn opendp_measurements__make_randomized_response_bool(
    prob: f64,
    constant_time: c_bool,
) -> FfiResult<*mut AnyMeasurement> {
    let constant_time = to_bool(constant_time);
    make_randomized_response_bool(prob, constant_time)
        .into_any()
        .into()
}

#[no_mangle]
pub extern "C" fn opendp_measurements__make_randomized_response(
    categories: *const AnyObject,
    prob: f64,
    T: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T: Hashable>(
        categories: *const AnyObject,
        prob: f64,
    ) -> Fallible<AnyMeasurement> {
        let categories = try_as_ref!(categories).downcast_ref::<Vec<T>>()?.clone();
        make_randomized_response::<T>(HashSet::from_iter(categories.into_iter()), prob).into_any()
    }
    let T_ = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T_, @hashable)
    ], (categories, prob))
    .into()
}
