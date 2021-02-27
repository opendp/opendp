use std::os::raw::c_char;

use num::NumCast;

use opendp::meas::{MakeMeasurement1};
use opendp::meas::gaussian::GaussianMechanism;
use opendp::meas::laplace::{VectorLaplaceMechanism, LaplaceMechanism};

use crate::core::FfiMeasurement;
use crate::util;
use crate::util::TypeArgs;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace(type_args: *const c_char, sigma: f64) -> *mut FfiMeasurement {
    fn monomorphize<T>(sigma: f64) -> *mut FfiMeasurement where
        T: 'static + Copy + NumCast {
        let measurement = LaplaceMechanism::<T>::make(sigma).unwrap();
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (sigma))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace_vec(type_args: *const c_char, sigma: f64) -> *mut FfiMeasurement {
    fn monomorphize<T>(sigma: f64) -> *mut FfiMeasurement where
        T: 'static + Copy + NumCast {
        let measurement = VectorLaplaceMechanism::<T>::make(sigma).unwrap();
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (sigma))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian(type_args: *const c_char, sigma: f64) -> *mut FfiMeasurement {
    fn monomorphize<T>(sigma: f64) -> *mut FfiMeasurement where
        T: 'static + Copy + NumCast {
        let measurement = GaussianMechanism::<T>::make(sigma).unwrap();
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (sigma))
}

#[no_mangle]
pub extern "C" fn opendp_meas__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "FfiMeasurement *" },
    { "name": "make_base_laplace_vec", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "FfiMeasurement *" },
    { "name": "make_base_gaussian", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "FfiMeasurement *" }
]
}"#;
    util::bootstrap(spec)
}
