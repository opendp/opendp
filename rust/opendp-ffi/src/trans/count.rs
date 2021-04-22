use std::hash::Hash;
use std::ops::AddAssign;
use std::os::raw::{c_char, c_uint};

use num::{Integer, One, Zero};
use num::traits::FloatConst;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{CountByCategoriesConstant, CountByConstant, make_count, make_count_by, make_count_by_categories};

use crate::core::{FfiObject, FfiResult, FfiTransformation};
use crate::util::{parse_type_args, Type};

#[no_mangle]
pub extern "C" fn opendp_trans__make_count(type_args: *const c_char) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<MI, MO, T: 'static>() -> FfiResult<*mut FfiTransformation>
        where MI: 'static + DatasetMetric<Distance=u32> + Clone,
              MO: 'static + SensitivityMetric<Distance=u32> + Clone {
        make_count::<MI, MO, T>().into()
    }
    let type_args = try_!(parse_type_args(type_args, 3));
    dispatch!(monomorphize, [
        (type_args[0], [SymmetricDistance, HammingDistance]),
        (type_args[1], [L1Sensitivity<u32>, L2Sensitivity<u32>]),
        (type_args[2], @primitives)
    ], ())
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by_categories(type_args: *const c_char, categories: *const FfiObject) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<QO>(type_args: Vec<Type>, categories: *const FfiObject) -> FfiResult<*mut FfiTransformation>
        where QO: DistanceConstant + FloatConst + One {
        fn monomorphize2<MI, MO, TI, TO>(categories: *const FfiObject) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  TI: 'static + Eq + Hash + Clone,
                  TO: 'static + Integer + Zero + One + AddAssign,
                  MO::Distance: DistanceConstant + FloatConst + One,
                  (MI, MO): CountByCategoriesConstant<MI, MO> {
            let categories = try_as_ref!(categories as *const Vec<TI>).clone();
            make_count_by_categories::<MI, MO, TI, TO>(categories).into()
        }
        dispatch!(monomorphize2, [
            (type_args[0], [HammingDistance, SymmetricDistance]),
            (type_args[1], [L1Sensitivity<QO>, L2Sensitivity<QO>]),
            (type_args[2], @hashable),
            (type_args[3], @integers)
        ], (categories))
    }
    let type_args = try_!(parse_type_args(type_args, 4));
    let type_output_distance = try_!(type_args[1].get_sensitivity_distance());
    dispatch!(monomorphize, [(type_output_distance, @floats)], (type_args, categories))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by(type_args: *const c_char, n: c_uint) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<QO>(type_args: Vec<Type>, n: usize) -> FfiResult<*mut FfiTransformation>
        where QO: DistanceConstant + FloatConst + One {
        fn monomorphize2<MI, MO, TI, TO>(n: usize) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  TI: 'static + Eq + Hash + Clone,
                  TO: 'static + Integer + Zero + One + AddAssign,
                  MO::Distance: DistanceConstant + FloatConst + One,
                  (MI, MO): CountByConstant<MI, MO> {
            make_count_by::<MI, MO, TI, TO>(n).into()
        }
        dispatch!(monomorphize2, [
            (type_args[0], [HammingDistance, SymmetricDistance]),
            (type_args[1], [L1Sensitivity<QO>, L2Sensitivity<QO>]),
            (type_args[2], @hashable),
            (type_args[3], @integers)
        ], (n))
    }
    let n = n as usize;
    let type_args: Vec<Type> = try_!(parse_type_args(type_args, 4));
    let type_output = try_!(type_args[1].get_sensitivity_distance());
    dispatch!(monomorphize, [(type_output, @floats)], (type_args, n))
}
