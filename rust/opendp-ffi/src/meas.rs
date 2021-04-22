use std::hash::Hash;
use std::ops::{AddAssign};
use std::os::raw::{c_char, c_void};

use num::{CheckedAdd, CheckedSub, Float, Integer, NumCast, One, Zero};

use opendp::core::SensitivityMetric;
use opendp::dist::{L1Sensitivity, L2Sensitivity};
use opendp::err;
use opendp::meas::*;
use opendp::samplers::{CastRug, SampleGaussian, SampleGeometric, SampleLaplace};
use opendp::traits::DistanceCast;

use crate::core::{FfiMeasurement, FfiResult};
use crate::util;
use crate::util::{parse_type_args, Type};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace(type_args: *const c_char, scale: *const c_void) -> FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
        let scale = *try_as_ref!(scale as *const T);
        make_base_laplace::<T>(scale).into()
    }
    let type_args = try_!(parse_type_args(type_args, 1));
    dispatch!(monomorphize, [(type_args[0], @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace_vec(type_args: *const c_char, scale: *const c_void) -> FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
        let scale = *try_as_ref!(scale as *const T);
        make_base_laplace_vec::<T>(scale).into()
    }
    let type_args = try_!(parse_type_args(type_args, 1));
    dispatch!(monomorphize, [(type_args[0], @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian(type_args: *const c_char, scale: *const c_void) -> FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement> where
        T: 'static + Clone + SampleGaussian + Float {
        let scale = *try_as_ref!(scale as *const T);
        make_base_gaussian::<T>(scale).into()
    }
    let type_args = try_!(parse_type_args(type_args, 1));
    dispatch!(monomorphize, [(type_args[0], @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian_vec(type_args: *const c_char, scale: *const c_void) -> FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T>(scale: *const c_void) -> FfiResult<*mut FfiMeasurement> where
        T: 'static + Clone + SampleGaussian + Float {
        let scale = *try_as_ref!(scale as *const T);
        make_base_gaussian_vec::<T>(scale).into()
    }
    let type_args = try_!(parse_type_args(type_args, 1));
    dispatch!(monomorphize, [(type_args[0], @floats)], (scale))
}


#[no_mangle]
pub extern "C" fn opendp_meas__make_base_simple_geometric(type_args: *const c_char, scale: *const c_void, min: *const c_void, max: *const c_void) -> FfiResult<*mut FfiMeasurement> {
    fn monomorphize<T, QO>(scale: *const c_void, min: *const c_void, max: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where T: 'static + Clone + SampleGeometric + CheckedSub<Output=T> + CheckedAdd<Output=T> + DistanceCast + Zero,
              QO: 'static + Float + DistanceCast, f64: From<QO> {
        let scale = *try_as_ref!(scale as *const QO);
        let min = try_as_ref!(min as *const T).clone();
        let max = try_as_ref!(max as *const T).clone();
        make_base_geometric::<T, QO>(scale, min, max).into()
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    dispatch!(monomorphize, [(type_args[0], @integers), (type_args[1], @floats)], (scale, min, max))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_stability(type_args: *const c_char, n: usize, scale: *const c_void, threshold: *const c_void) -> FfiResult<*mut FfiMeasurement> {
    fn monomorphize<TIC, TOC>(type_args: Vec<Type>, n: usize, scale: *const c_void, threshold: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where TIC: 'static + Integer + Zero + One + AddAssign + Clone + NumCast,
              TOC: 'static + PartialOrd + Clone + NumCast + Float + CastRug {

        fn monomorphize2<MI, TIK, TIC, TOC>(n: usize, scale: TOC, threshold: TOC) -> FfiResult<*mut FfiMeasurement>
            where MI: 'static + SensitivityMetric<Distance=TOC> + BaseStabilityNoise<TOC>,
                  TIK: 'static + Eq + Hash + Clone,
                  TIC: 'static + Integer + Zero + One + AddAssign + Clone + NumCast,
                  TOC: 'static + Clone + NumCast + PartialOrd + Float + CastRug {
            make_base_stability::<MI, TIK, TIC, TOC>(n, scale, threshold).into()
        }
        let scale = *try_as_ref!(scale as *const TOC);
        let threshold = *try_as_ref!(threshold as *const TOC);
        dispatch!(monomorphize2, [
            (type_args[0], [L1Sensitivity<TOC>, L2Sensitivity<TOC>]),
            (type_args[1], @hashable),
            (type_args[2], [TIC]),
            (type_args[3], [TOC])
        ], (n, scale, threshold))
    }
    let type_args = try_!(parse_type_args(type_args, 4));
    dispatch!(monomorphize, [
        (type_args[2], @integers),
        (type_args[3], @floats)
    ], (type_args, n, scale, threshold))
}

#[no_mangle]
pub extern "C" fn opendp_meas__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["void *", "scale"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_laplace_vec", "args": [ ["const char *", "selector"], ["void *", "scale"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_gaussian", "args": [ ["const char *", "selector"], ["void *", "scale"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_simple_geometric", "args": [ ["const char *", "selector"], ["void *", "scale"], ["void *", "min"], ["void *", "max"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_stability", "args": [ ["const char *", "selector"], ["unsigned int", "n"], ["void *", "scale"], ["void *", "threshold"] ], "ret": "FfiResult<FfiMeasurement *>" }
]
}"#;
    util::bootstrap(spec)
}
