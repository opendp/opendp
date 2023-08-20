use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::err;
use crate::ffi::any::{AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::{c_bool, to_bool, Type};
use crate::measurements::{make_randomized_response, make_randomized_response_bool};
use crate::traits::samplers::SampleBernoulli;
use crate::traits::{Float, Hashable};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_randomized_response_bool(
    prob: *const c_void,
    constant_time: c_bool,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<QO>(prob: *const c_void, constant_time: bool) -> FfiResult<*mut AnyMeasurement>
    where
        bool: SampleBernoulli<QO>,
        QO: Float,
    {
        let prob = *try_as_ref!(prob as *const QO);
        make_randomized_response_bool::<QO>(prob, constant_time).into_any()
    }
    let QO = try_!(Type::try_from(QO));
    let constant_time = to_bool(constant_time);
    dispatch!(monomorphize, [
        (QO, @floats)
    ], (prob, constant_time))
}

#[no_mangle]
pub extern "C" fn opendp_measurements__make_randomized_response(
    categories: *const AnyObject,
    prob: *const c_void,
    constant_time: c_bool,
    T: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QO>(
        categories: *const AnyObject,
        prob: *const c_void,
        constant_time: bool,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: Hashable,
        bool: SampleBernoulli<QO>,
        QO: Float,
    {
        let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<T>>()).clone();
        let prob = *try_as_ref!(prob as *const QO);
        make_randomized_response::<T, QO>(
            HashSet::from_iter(categories.into_iter()),
            prob,
            constant_time,
        )
        .into_any()
    }
    let T = try_!(Type::try_from(T));
    let QO = try_!(Type::try_from(QO));
    let constant_time = to_bool(constant_time);
    dispatch!(monomorphize, [
        (T, @hashable),
        (QO, @floats)
    ], (categories, prob, constant_time))
}
