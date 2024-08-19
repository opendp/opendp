use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::error::Fallible;
use crate::ffi::any::{AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::{c_bool, to_bool, Type};
use crate::measurements::{make_randomized_response, make_randomized_response_bool};
use crate::traits::{ExactIntCast, Float, FloatBits, Hashable};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_randomized_response_bool(
    prob: *const c_void,
    constant_time: c_bool,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<QO>(prob: *const c_void, constant_time: bool) -> Fallible<AnyMeasurement>
    where
        QO: Float,
        <QO as FloatBits>::Bits: ExactIntCast<usize>,
        usize: ExactIntCast<<QO as FloatBits>::Bits>,
    {
        let prob = *try_as_ref!(prob as *const QO);
        make_randomized_response_bool::<QO>(prob, constant_time).into_any()
    }
    let QO = try_!(Type::try_from(QO));
    let constant_time = to_bool(constant_time);
    dispatch!(monomorphize, [
        (QO, @floats)
    ], (prob, constant_time))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_measurements__make_randomized_response(
    categories: *const AnyObject,
    prob: *const c_void,
    T: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QO>(
        categories: *const AnyObject,
        prob: *const c_void,
    ) -> Fallible<AnyMeasurement>
    where
        T: Hashable,
        QO: Float,
        <QO as FloatBits>::Bits: ExactIntCast<usize>,
        usize: ExactIntCast<<QO as FloatBits>::Bits>,
    {
        let categories = try_as_ref!(categories).downcast_ref::<Vec<T>>()?.clone();
        let num_categories = categories.len();
        let categories_set = HashSet::from_iter(categories.into_iter());
        if categories_set.len() != num_categories {
            return fallible!(MakeMeasurement, "all categories must be unique");
        }
        let prob = *try_as_ref!(prob as *const QO);
        make_randomized_response::<T, QO>(categories_set, prob).into_any()
    }
    let T = try_!(Type::try_from(T));
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @hashable),
        (QO, @floats)
    ], (categories, prob))
    .into()
}
