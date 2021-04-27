use std::os::raw::{c_char, c_void};

use num::One;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::traits::{CastFrom, DistanceConstant};
use opendp::trans::{make_cast_vec, make_clamp, make_clamp_vec, make_identity};

use crate::core::{FfiResult, FfiTransformation};
use crate::util::{parse_type_args, Type, TypeContents};

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(type_args: *const c_char) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize_scalar<M, T>() -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric,
              M::Distance: DistanceConstant + One,
              T: 'static + Clone {
        make_identity::<AllDomain<T>, M>(AllDomain::<T>::new(), M::default()).into()
    }
    fn monomorphize_vec<M, T>() -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric,
              M::Distance: DistanceConstant + One,
              T: 'static + Clone {
        make_identity::<VectorDomain<AllDomain<T>>, M>(VectorDomain::new(AllDomain::<T>::new()), M::default()).into()
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    match &type_args[1].contents {
        TypeContents::VEC(element_id) => dispatch!(monomorphize_vec, [
            (type_args[0], @dist_dataset),
            (try_!(Type::of_id(element_id)), @primitives)
        ], ()),
        _ => dispatch!(monomorphize_scalar, [
            (type_args[0], @dist_dataset),
            (&type_args[1], @primitives)
        ], ())
    }
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp_vec(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, T>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric + Clone,
              T: 'static + Copy + PartialOrd,
              M::Distance: DistanceConstant + One {
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        make_clamp_vec::<M, T>(lower, upper).into()
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    dispatch!(monomorphize, [(type_args[0], @dist_dataset), (type_args[1], @numbers)], (lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp_scalar(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<T, Q>(type_args: Vec<Type>, lower: *const c_void, upper: *const c_void) -> FfiResult<*mut FfiTransformation>
        where T: 'static + Clone + PartialOrd,
              Q: DistanceConstant + One {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();

        fn monomorphize2<M, T>(lower: T, upper: T) -> FfiResult<*mut FfiTransformation>
            where M: 'static + SensitivityMetric,
                  T: 'static + Clone + PartialOrd,
                  M::Distance: DistanceConstant + One {
            make_clamp::<M, T>(lower, upper).into()
        }
        dispatch!(monomorphize2, [
            (type_args[0], [L1Sensitivity<Q>, L2Sensitivity<Q>]),
            (type_args[1], [T])
        ], (lower, upper))
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    let type_output_distance = try_!(type_args[0].get_sensitivity_distance());
    dispatch!(monomorphize, [
        (type_args[1], @numbers),
        (type_output_distance, @numbers)
    ], (type_args, lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_vec(type_args: *const c_char) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, TI, TO>() -> FfiResult<*mut FfiTransformation> where
        M: 'static + DatasetMetric<Distance=u32>,
        TI: 'static + Clone,
        TO: 'static + CastFrom<TI> + Default {
        make_cast_vec::<M, TI, TO>().into()
    }
    let type_args = try_!(parse_type_args(type_args, 3));
    dispatch!(monomorphize, [(type_args[0], @dist_dataset), (type_args[1], @primitives), (type_args[2], @primitives)], ())
}

// #[no_mangle]
// pub extern "C" fn opendp_trans__make_cast(type_args: *const c_char) -> FfiResult<*mut FfiTransformation> {
//     fn monomorphize<TI, TO>(type_args: Vec<Type>) -> FfiResult<*mut FfiTransformation>
//         where TI: 'static + Clone + DistanceCast,
//               TO: 'static + CastFrom<TI> + Default + DistanceCast + One + Div<Output=TO> + Mul<Output=TO> + PartialOrd {
//
//         fn monomorphize2<MI, MO, TI, TO>() -> FfiResult<*mut FfiTransformation>
//             where MI: 'static + SensitivityMetric<Distance=TI>,
//                   MO: 'static + SensitivityMetric<Distance=TO>,
//                   TI: 'static + Clone + DistanceCast,
//                   TO: 'static + CastFrom<TI> + Default + DistanceCast + One + Div<Output=TO> + Mul<Output=TO> + PartialOrd {
//             let transformation = trans::manipulation::Cast::<MI, MO, TI, TO>();
//             FfiResult::new(transformation.map(FfiTransformation::new_from_types))
//         }
//         dispatch!(monomorphize2, [
//             (type_args[0], [L1Sensitivity<TI>, L2Sensitivity<TI>]),
//             (type_args[1], [L1Sensitivity<TO>, L2Sensitivity<TO>]),
//             (type_args[2], [TI]), (type_args[3], [TO])
//         ], ())
//     }
//     let type_args = try_!(parse_type_args(type_args, 4));
//     dispatch!(monomorphize, [(type_args[2], @numbers), (type_args[3], @numbers)], (type_args))
// }
