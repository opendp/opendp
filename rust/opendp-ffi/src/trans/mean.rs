use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use num::Float;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{BoundedMeanConstant, make_bounded_mean};

use crate::core::{FfiResult, FfiTransformation};
use crate::util::{Type, parse_type_args};

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_mean(type_args: *const c_char, lower: *const c_void, upper: *const c_void, length: c_uint) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<T>(type_args: Vec<Type>, lower: *const c_void, upper: *const c_void, length: usize) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T> + Float,
              for<'a> T: Sum<&'a T> {
        fn monomorphize2<MI, MO>(lower: MO::Distance, upper: MO::Distance, length: usize) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Float,
                  for<'a> MO::Distance: Sum<&'a MO::Distance>,
                  (MI, MO): BoundedMeanConstant<MI, MO> {
            make_bounded_mean::<MI, MO>(lower, upper, length).into()
        }
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        dispatch!(monomorphize2, [
            (type_args[0], [HammingDistance, SymmetricDistance]),
            (type_args[1], [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, length))
    }
    let length = length as usize;
    let type_args = try_!(parse_type_args(type_args, 2));
    let type_output = try_!(type_args[1].get_sensitivity_distance());
    dispatch!(monomorphize, [(type_output, @floats)], (type_args, lower, upper, length))
}