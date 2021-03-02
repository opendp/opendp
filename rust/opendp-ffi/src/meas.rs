use std::convert::TryFrom;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::AddAssign;
use std::os::raw::c_char;

use num::{Integer, NumCast, One, Zero};

use opendp::dist::{L1Sensitivity, HammingDistance};
use opendp::meas::{MakeMeasurement1, MakeMeasurement2, MakeMeasurement3};
use opendp::meas::gaussian::GaussianMechanism;
use opendp::meas::laplace::{LaplaceMechanism, VectorLaplaceMechanism};
use opendp::meas::stability::{BaseStability, StabilityMechanism};

use crate::core::FfiMeasurement;
use crate::util;
use crate::util::TypeArgs;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace(type_args: *const c_char, sigma: f64) -> *mut FfiMeasurement {
    fn monomorphize<T>(sigma: f64) -> *mut FfiMeasurement where
        T: 'static + Copy + NumCast {
        let measurement = LaplaceMechanism::make(sigma);
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], [f64])], (sigma))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_laplace_vec(type_args: *const c_char, length: usize, sigma: f64) -> *mut FfiMeasurement {
    fn monomorphize<T>(length: usize, sigma: f64) -> *mut FfiMeasurement where
        T: 'static + Copy + NumCast {
        let measurement = VectorLaplaceMechanism::make(length, sigma);
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], [f64])], (length, sigma))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_gaussian(type_args: *const c_char, sigma: f64) -> *mut FfiMeasurement {
    fn monomorphize<T>(sigma: f64) -> *mut FfiMeasurement where
        T: 'static + Copy + NumCast {
        let measurement = GaussianMechanism::<T>::make(sigma);
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (sigma))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_stability_l1(type_args: *const c_char, n: usize, sigma: f64, threshold: f64) -> *mut FfiMeasurement {
    fn monomorphize<TIK, TIC>(n: usize, sigma: f64, threshold: f64) -> *mut FfiMeasurement
        where TIK: 'static + Eq + Hash + Clone,
              TIC: Integer + Zero + One + AddAssign + Clone,
              f64: TryFrom<TIC>,
              <f64 as TryFrom<TIC>>::Error: Debug {
        let measurement = BaseStability::<L1Sensitivity<f64>, u32, u32>::make(n, sigma, threshold);
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 2);
    dispatch!(monomorphize, [(type_args.0[0], @integers), (type_args.0[1], @integers)], (n, sigma, threshold))
}

#[no_mangle]
pub extern "C" fn opendp_meas__make_stability_mechanism_l1(type_args: *const c_char, n: usize, sigma: f64, threshold: f64) -> *mut FfiMeasurement {
    fn monomorphize<TIK, TIC>(n: usize, sigma: f64, threshold: f64) -> *mut FfiMeasurement
        where TIK: 'static + Eq + Hash + Clone,
              TIC: 'static + Integer + Zero + One + AddAssign + Clone,
              f64: TryFrom<TIC>,
              <f64 as TryFrom<TIC>>::Error: Debug {
        let measurement = StabilityMechanism::<HammingDistance, TIK, TIC>::make(n, sigma, threshold);
        FfiMeasurement::new_from_types(measurement)
    }
    let type_args = TypeArgs::expect(type_args, 2);
    dispatch!(monomorphize, [(type_args.0[0], @hashable), (type_args.0[1], @integers)], (n, sigma, threshold))
}

#[no_mangle]
pub extern "C" fn opendp_meas__bootstrap() -> *const c_char {
    let spec =
        r#"{
"functions": [
    { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "void *" },
    { "name": "make_base_laplace_vec", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "void *" },
    { "name": "make_base_gaussian", "args": [ ["const char *", "selector"], ["double", "sigma"] ], "ret": "void *" },
    { "name": "make_base_stability_l1", "args": [ ["const char *", "selector"], ["size_t", "n"], ["double", "sigma"], ["double", "threshold"] ], "ret": "void *" },
    { "name": "make_stability_mechanism_l1", "args": [ ["const char *", "selector"], ["size_t", "n"], ["double", "sigma"], ["double", "threshold"] ], "ret": "void *" }
]
}"#;
    util::bootstrap(spec)
}
