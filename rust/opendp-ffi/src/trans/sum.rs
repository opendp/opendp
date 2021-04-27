use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::{Abs, DistanceConstant};
use opendp::trans::{BoundedSumConstant, make_bounded_sum, make_bounded_sum_n};

use crate::core::{FfiResult, FfiTransformation};
use crate::util::{Type, parse_type_args};

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<T>(type_args: Vec<Type>, lower: *const c_void, upper: *const c_void) -> FfiResult<*mut FfiTransformation>
        where for<'a> T: DistanceConstant + Sub<Output=T> + Abs + Sum<&'a T> {
        fn monomorphize2<MI, MO>(lower: MO::Distance, upper: MO::Distance) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  for<'a> MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Abs + Sum<&'a MO::Distance>,
                  (MI, MO): BoundedSumConstant<MI, MO> {
            make_bounded_sum::<MI, MO>(lower, upper).into()
        }
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        dispatch!(monomorphize2, [
            (type_args[0], [HammingDistance, SymmetricDistance]),
            (type_args[1], [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper))
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    let type_output = try_!(type_args[1].get_sensitivity_distance());
    dispatch!(monomorphize, [(type_output, @numbers)], (type_args, lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum_n(type_args: *const c_char, lower: *const c_void, upper: *const c_void, n: c_uint) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<T>(type_args: Vec<Type>, lower: *const c_void, upper: *const c_void, n: usize) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T>,
              for<'a> T: Sum<&'a T> {
        fn monomorphize2<MO>(lower: MO::Distance, upper: MO::Distance, n: usize) -> FfiResult<*mut FfiTransformation>
            where MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance>,
                  for<'a> MO::Distance: Sum<&'a MO::Distance> {
            make_bounded_sum_n::<MO>(lower, upper, n).into()
        }
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        dispatch!(monomorphize2, [
            (type_args[0], [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, n))
    }
    let n = n as usize;
    let type_args = try_!(parse_type_args(type_args, 1));
    let type_output = try_!(type_args[0].get_sensitivity_distance());
    dispatch!(monomorphize, [(type_output, @numbers)], (type_args, lower, upper, n))
}