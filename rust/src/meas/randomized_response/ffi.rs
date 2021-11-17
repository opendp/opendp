use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::Hash;
use std::iter::FromIterator;
use std::os::raw::{c_char, c_void};

use num::Float;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::dist::IntDistance;
use crate::err;
use crate::ffi::any::{AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::{c_bool, to_bool, Type};
use crate::meas::{make_randomized_response, make_randomized_response_bool};
use crate::samplers::SampleBernoulli;
use crate::traits::{CheckNull, DistanceConstant, ExactIntCast, InfCast, InfLn, InfSub};

#[no_mangle]
pub extern "C" fn opendp_meas__make_randomized_response_bool(
    prob: *const c_void,
    constant_time: c_bool,
    Q: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<Q>(prob: *const c_void, constant_time: bool) -> FfiResult<*mut AnyMeasurement>
        where bool: SampleBernoulli<Q>,
              Q: 'static + Float + ExactIntCast<IntDistance> + DistanceConstant<IntDistance> + InfSub + InfLn,
              IntDistance: InfCast<Q> {
        let prob = *try_as_ref!(prob as *const Q);
        make_randomized_response_bool::<Q>(prob, constant_time).into_any()
    }
    let Q = try_!(Type::try_from(Q));
    let constant_time = to_bool(constant_time);
    dispatch!(monomorphize, [
        (Q, @floats)
    ], (prob, constant_time))
}


#[no_mangle]
pub extern "C" fn opendp_meas__make_randomized_response(
    categories: *const AnyObject,
    prob: *const c_void,
    constant_time: c_bool,
    T: *const c_char,
    Q: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, Q>(
        categories: *const AnyObject, prob: *const c_void,
        constant_time: bool,
    ) -> FfiResult<*mut AnyMeasurement>
        where T: 'static + Clone + Eq + Hash + CheckNull,
              bool: SampleBernoulli<Q>,
              Q: 'static + Float + ExactIntCast<usize> + DistanceConstant<IntDistance> + InfSub + InfLn,
              IntDistance: InfCast<Q> {
        let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<T>>()).clone();
        let prob = *try_as_ref!(prob as *const Q);
        make_randomized_response::<T, Q>(
            HashSet::from_iter(categories.into_iter()),
            prob, constant_time).into_any()
    }
    let T = try_!(Type::try_from(T));
    let Q = try_!(Type::try_from(Q));
    let constant_time = to_bool(constant_time);
    dispatch!(monomorphize, [
        (T, @hashable),
        (Q, @floats)
    ], (categories, prob, constant_time))
}