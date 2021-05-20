use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::err;
use opendp::meas::{make_base_gaussian, make_base_gaussian_vec};
use opendp::samplers::SampleGaussian;

use crate::core::{FfiMeasurement, FfiResult};
use crate::util::Type;
use std::convert::TryFrom;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian(
    scale: *const c_void, T: *const c_char
) -> FfiResult<*mut FfiMeasurement> {

    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement> where
        T: 'static + Clone + SampleGaussian + Float {
        let scale = *try_as_ref!(scale as *const T);
        make_base_gaussian::<T>(scale).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian_vec(
    scale: *const c_void, T: *const c_char
) -> FfiResult<*mut FfiMeasurement> {

    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement> where
        T: 'static + Clone + SampleGaussian + Float {
        let scale = *try_as_ref!(scale as *const T);
        make_base_gaussian_vec::<T>(scale).into()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @floats)], (scale))
}
