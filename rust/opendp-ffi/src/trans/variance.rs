use std::iter::Sum;
use std::ops::{Add, Div, Sub};
use std::os::raw::{c_char, c_uint};

use num::{Float, One, Zero};

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{BoundedCovarianceConstant, BoundedVarianceConstant, make_bounded_covariance, make_bounded_variance};

use crate::core::{FfiObject, FfiResult, FfiTransformation};
use crate::util::{parse_type_args, Type};

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_variance(
    type_args: *const c_char,
    lower: *const FfiObject, upper: *const FfiObject,
    length: c_uint, ddof: c_uint,
) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<T>(
        type_args: Vec<Type>,
        lower: *const FfiObject, upper: *const FfiObject,
        length: usize, ddof: usize,
    ) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T> + Float + for<'a> Sum<&'a T> + Sum<T>,
              for<'a> &'a T: Sub<Output=T> {
        fn monomorphize2<MI, MO>(lower: MO::Distance, upper: MO::Distance, length: usize, ddof: usize) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Float + for<'a> Sum<&'a MO::Distance> + Sum<MO::Distance>,
                  for<'a> &'a MO::Distance: Sub<Output=MO::Distance>,
                  (MI, MO): BoundedVarianceConstant<MI, MO> {
            make_bounded_variance::<MI, MO>(lower, upper, length, ddof).into()
        }
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        dispatch!(monomorphize2, [
            (type_args[0], [HammingDistance, SymmetricDistance]),
            (type_args[1], [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, length, ddof))
    }
    let length = length as usize;
    let ddof = ddof as usize;

    let type_args = try_!(parse_type_args(type_args, 2));
    let type_output = try_!(type_args[1].get_sensitivity_distance());
    dispatch!(monomorphize, [(type_output, @floats)], (type_args, lower, upper, length, ddof))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_covariance(
    type_args: *const c_char,
    lower: *const FfiObject,
    upper: *const FfiObject,
    length: c_uint, ddof: c_uint,
) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<T>(
        type_args: Vec<Type>,
        lower: *const FfiObject,
        upper: *const FfiObject,
        length: usize, ddof: usize,
    ) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T> + Sum<T> + Zero + One,
              for<'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
              for<'a> &'a T: Sub<Output=T> {
        fn monomorphize2<MI, MO>(
            lower: (MO::Distance, MO::Distance),
            upper: (MO::Distance, MO::Distance),
            length: usize, ddof: usize,
        ) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Sum<MO::Distance> + Zero + One,
                  for<'a> MO::Distance: Div<&'a MO::Distance, Output=MO::Distance> + Add<&'a MO::Distance, Output=MO::Distance>,
                  for<'a> &'a MO::Distance: Sub<Output=MO::Distance>,
                  (MI, MO): BoundedCovarianceConstant<MI, MO> {
            make_bounded_covariance::<MI, MO>(lower, upper, length, ddof).into()
        }
        let lower = try_as_ref!(lower).as_ref::<(T, T)>().clone();
        let upper = try_as_ref!(upper).as_ref::<(T, T)>().clone();
        dispatch!(monomorphize2, [
            (type_args[0], [HammingDistance, SymmetricDistance]),
            (type_args[1], [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, length, ddof))
    }
    let length = length as usize;
    let ddof = ddof as usize;

    let type_args = try_!(parse_type_args(type_args, 3));
    dispatch!(monomorphize, [(type_args[2], @floats)], (type_args, lower, upper, length, ddof))
}
