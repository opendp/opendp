use std::convert::TryFrom;
use std::os::raw::{c_char, c_void, c_int};

use num::{Integer};

use opendp::err;
use opendp::meas::{make_alp_histogram_post_process, make_alp_histogram_parameterized};
use opendp::samplers::CastInternalReal;
use opendp::traits::DistanceCast;

use crate::any::AnyMeasurement;
use crate::core::{FfiResult, IntoAnyQueryableMeasurementFfiResultExt};
use crate::util::Type;
use std::hash::Hash;

#[no_mangle]
pub extern "C" fn opendp_meas__make_alp_histogram(
    n: c_int, alpha: *const c_void, scale: *const c_void, beta: *const c_void, size_factor: u32,
    K: *const c_char, C: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<K, C, T>(
        n: usize, alpha: *const c_void, scale: *const c_void, beta: *const c_void, size_factor: u32,
    ) -> FfiResult<*mut AnyMeasurement>
        where K: 'static + Eq + Hash + Clone,
              C: 'static + Clone + Integer + DistanceCast + Hash,
              T: 'static + num::Float + DistanceCast + CastInternalReal {
        let alpha = try_as_ref!(alpha as *const T).clone();
        let scale = try_as_ref!(scale as *const T).clone();
        let beta = try_as_ref!(beta as *const C).clone();
        let meas = try_!(make_alp_histogram_parameterized::<K, C, T>(
            n, alpha, scale, beta, size_factor));
        // this is chained immediately because below doesn't work with an AnyMeasurement arg
        make_alp_histogram_post_process(&meas).into_any_queryable()
    }
    let n = n as usize;
    let size_factor = size_factor as u32;
    dispatch!(monomorphize, [
        (try_!(Type::try_from(K)), @hashable),
        (try_!(Type::try_from(C)), @integers),
        (try_!(Type::try_from(T)), @floats)
    ], (n, alpha, scale, beta, size_factor))
}

