use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::meas::{MakeMeasurement1, SampleGaussian, SampleLaplace};
use opendp::meas::gaussian::BaseGaussian;
use opendp::meas::laplace::{BaseVectorLaplace, BaseLaplace};

use crate::core::FfiMeasurement;
use crate::util;
use crate::util::TypeArgs;
use opendp::traits::DistanceCast;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace(type_args: *const c_char, scale: *const c_void) -> *mut FfiMeasurement {
    fn monomorphize<T>(scale: *const c_void) -> *mut FfiMeasurement
        where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
        let scale = util::as_ref(scale as *const T).clone();
        let measurement = BaseLaplace::<T>::make(scale).unwrap();
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace_vec(type_args: *const c_char, scale: *const c_void) -> *mut FfiMeasurement {
    fn monomorphize<T>(scale: *const c_void) -> *mut FfiMeasurement
        where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
        let scale = util::as_ref(scale as *const T).clone();
        let measurement = BaseVectorLaplace::<T>::make(scale).unwrap();
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian(type_args: *const c_char, scale: *const c_void) -> *mut FfiMeasurement {
    fn monomorphize<T>(scale: *const c_void) -> *mut FfiMeasurement where
        T: 'static + Copy + SampleGaussian + Float {
        let scale = util::as_ref(scale as *const T).clone();
        let measurement = BaseGaussian::<T>::make(scale).unwrap();
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian_vec(type_args: *const c_char, scale: *const c_void) -> *mut FfiMeasurement {
    fn monomorphize<T>(scale: *const c_void) -> *mut FfiMeasurement where
        T: 'static + Copy + SampleGaussian + Float {
        let scale = util::as_ref(scale as *const T).clone();
        let measurement = BaseGaussian::<T>::make(scale).unwrap();
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @floats)], (scale))
}

#[no_mangle]
pub extern "C" fn opendp_meas__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["double", "scale"] ], "ret": "FfiMeasurement *" },
    { "name": "make_base_laplace_vec", "args": [ ["const char *", "selector"], ["double", "scale"] ], "ret": "FfiMeasurement *" },
    { "name": "make_base_gaussian", "args": [ ["const char *", "selector"], ["double", "scale"] ], "ret": "FfiMeasurement *" }
]
}"#;
    util::bootstrap(spec)
}
