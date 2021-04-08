use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Div, Mul, Sub};
use std::os::raw::{c_char, c_uint, c_void};
use std::str::FromStr;

use num::One;

use opendp::core::{DatasetMetric, Metric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::dom::AllDomain;
use opendp::traits::{Abs, CastFrom, DistanceCast};
use opendp::trans::{BoundedSum, BoundedSumStability, MakeTransformation0, MakeTransformation1, MakeTransformation2, MakeTransformation3, manipulation};
use opendp::trans;

use crate::core::FfiTransformation;
use crate::util;
use crate::util::c_bool;
use crate::util::TypeArgs;

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(type_args: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<T: 'static + Clone>() -> *mut FfiTransformation {
        let transformation = manipulation::Identity::make(AllDomain::<T>::new(), HammingDistance::new()).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_lines(type_args: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<M>() -> *mut FfiTransformation
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        let transformation = trans::SplitLines::<M>::make().unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_series(type_args: *const c_char, impute: c_bool) -> *mut FfiTransformation {
    fn monomorphize<M, T>(impute: bool) -> *mut FfiTransformation
        where M: 'static + DatasetMetric<Distance=u32> + Clone,
              T: 'static + FromStr + Default,
              T::Err: Debug {
        let transformation = trans::ParseSeries::<T, M>::make(impute).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset), (type_args.0[1], @primitives)], (impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_records(type_args: *const c_char, separator: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<M>(separator: Option<&str>) -> *mut FfiTransformation
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        let transformation = trans::SplitRecords::<M>::make(separator).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let separator = util::to_option_str(separator);
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset)], (separator))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_create_dataframe(type_args: *const c_char, col_count: c_uint) -> *mut FfiTransformation {
    fn monomorphize<M>(col_names: Vec<i32>) -> *mut FfiTransformation
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        let transformation = trans::CreateDataFrame::<M>::make(col_names).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let col_names = (0..col_count as usize as i32).collect(); // TODO: pass Vec<T> over FFI
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset)], (col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_dataframe(type_args: *const c_char, separator: *const c_char, col_count: c_uint) -> *mut FfiTransformation {
    fn monomorphize<M>(separator: Option<&str>, col_names: Vec<i32>) -> *mut FfiTransformation
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        let transformation = trans::SplitDataFrame::<M>::make(separator, col_names).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let separator = util::to_option_str(separator);
    let col_names = (0..col_count as usize as i32).collect(); // TODO: pass Vec<T> over FFI
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset)], (separator, col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_column(type_args: *const c_char, key: *const c_void, impute: c_bool) -> *mut FfiTransformation {
    fn monomorphize<M, K, T>(key: *const c_void, impute: bool) -> *mut FfiTransformation where
        M: 'static + DatasetMetric<Distance=u32> + Clone,
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq + FromStr + Default,
        T::Err: Debug {
        let key = util::as_ref(key as *const K).clone();
        let transformation = trans::ParseColumn::<M, T>::make(key, impute).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 3);
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset), (type_args.0[1], @hashable), (type_args.0[2], @primitives)], (key, impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(type_args: *const c_char, key: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<M, K, T>(key: *const c_void) -> *mut FfiTransformation where
        M: 'static + DatasetMetric<Distance=u32> + Clone,
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq {
        let key = util::as_ref(key as *const K).clone();
        let transformation = trans::SelectColumn::<M, T>::make(key).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset), (type_args.0[1], @hashable), (type_args.0[2], @primitives)], (key))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp_vec(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<M, T>(lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation
        where M: 'static + Metric<Distance=u32> + Clone,
              T: 'static + Copy + PartialOrd {
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();
        let transformation = manipulation::Clamp::<M, Vec<T>, u32>::make(lower, upper).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 2);
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset), (type_args.0[1], @numbers)], (lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp_scalar(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<T, Q>(type_args: TypeArgs, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation
        where T: 'static + Clone + PartialOrd,
              Q: 'static + One + Mul<Output=Q> + Div<Output=Q> + PartialOrd + DistanceCast {
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();

        fn monomorphize2<M, T, Q>(lower: T, upper: T) -> *mut FfiTransformation
            where M: 'static + SensitivityMetric<Distance=Q>,
                  T: 'static + Clone + PartialOrd,
                  Q: 'static + One + Mul<Output=Q> + Div<Output=Q> + PartialOrd + DistanceCast {
            let transformation = trans::manipulation::Clamp::<M, T, Q>::make(lower, upper).unwrap();
            FfiTransformation::new_from_types(transformation)
        }
        dispatch!(monomorphize2, [
            (type_args.0[0], [L1Sensitivity<Q>, L2Sensitivity<Q>]),
            (type_args.0[2], [T]), (type_args.0[3], [Q])
        ], (lower, upper))
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [(type_args.0[2], @numbers), (type_args.0[3], @numbers)], (type_args, lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_cast_vec(type_args: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<M, TI, TO>() -> *mut FfiTransformation where
        M: 'static + DatasetMetric<Distance=u32>, TI: 'static + Clone, TO: 'static + CastFrom<TI> + Default {
        let transformation = trans::manipulation::Cast::<M, M, Vec<TI>, Vec<TO>>::make().unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [(type_args.0[0], @dist_dataset), (type_args.0[1], @primitives), (type_args.0[2], @primitives)], ())
}

// #[no_mangle]
// pub extern "C" fn opendp_trans__make_cast(type_args: *const c_char) -> *mut FfiTransformation {
//     fn monomorphize<TI, TO>(type_args: TypeArgs) -> *mut FfiTransformation
//         where TI: 'static + Clone + DistanceCast,
//               TO: 'static + CastFrom<TI> + Default + DistanceCast + One + Div<Output=TO> + Mul<Output=TO> + PartialOrd {
//
//         fn monomorphize2<MI, MO, TI, TO>() -> *mut FfiTransformation
//             where MI: 'static + SensitivityMetric<Distance=TI>,
//                   MO: 'static + SensitivityMetric<Distance=TO>,
//                   TI: 'static + Clone + DistanceCast,
//                   TO: 'static + CastFrom<TI> + Default + DistanceCast + One + Div<Output=TO> + Mul<Output=TO> + PartialOrd {
//             let transformation = trans::manipulation::Cast::<MI, MO, TI, TO>::make().unwrap();
//             FfiTransformation::new_from_types(transformation)
//         }
//         dispatch!(monomorphize2, [
//             (type_args.0[0], [L1Sensitivity<TI>, L2Sensitivity<TI>]),
//             (type_args.0[1], [L1Sensitivity<TO>, L2Sensitivity<TO>]),
//             (type_args.0[2], [TI]), (type_args.0[3], [TO])
//         ], ())
//     }
//     let type_args = TypeArgs::expect(type_args, 4);
//     dispatch!(monomorphize, [(type_args.0[2], @numbers), (type_args.0[3], @numbers)], (type_args))
// }

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<T>(type_args: TypeArgs, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation
        where T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast + Abs {

        fn monomorphize2<MI, MO, T>(lower: T, upper: T) -> *mut FfiTransformation
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric<Distance=T>,
                  T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast,
                  BoundedSum<MI, MO, T>: BoundedSumStability<MI, MO, T> {
            let transformation = trans::BoundedSum::<MI, MO, T>::make(lower, upper).unwrap();
            FfiTransformation::new_from_types(transformation)
        }
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();
        dispatch!(monomorphize2, [
            (type_args.0[0], [HammingDistance, SymmetricDistance]),
            (type_args.0[1], [L1Sensitivity<T>, L2Sensitivity<T>]),
            (type_args.0[2], [T])
        ], (lower, upper))
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [(type_args.0[2], @numbers)], (type_args, lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum_n(type_args: *const c_char, lower: *const c_void, upper: *const c_void, n: c_uint) -> *mut FfiTransformation {
    fn monomorphize<T>(type_args: TypeArgs, lower: *const c_void, upper: *const c_void, n: usize) -> *mut FfiTransformation
        where T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast + Abs {

        fn monomorphize2<MO, T>(lower: T, upper: T, n: usize) -> *mut FfiTransformation
            where MO: 'static + SensitivityMetric<Distance=T>,
                  T: 'static + Clone + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast {
            let transformation = trans::BoundedSum::<SymmetricDistance, MO, T>::make3(lower, upper, n).unwrap();
            FfiTransformation::new_from_types(transformation)
        }
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();
        dispatch!(monomorphize2, [
            (type_args.0[0], [L1Sensitivity<T>, L2Sensitivity<T>]),
            (type_args.0[1], [T])
        ], (lower, upper, n))
    }
    let n = n as usize;
    let type_args = TypeArgs::expect(type_args, 2);
    dispatch!(monomorphize, [(type_args.0[1], @numbers)], (type_args, lower, upper, n))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_count(type_args: *const c_char) -> *mut FfiTransformation {

    fn monomorphize<MI, MO, T: 'static>() -> *mut FfiTransformation
        where MI: 'static + DatasetMetric<Distance=u32> + Clone,
              MO: 'static + SensitivityMetric<Distance=u32> + Clone {
        let transformation = trans::Count::<MI, MO, T>::make().unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [
        (type_args.0[0], [SymmetricDistance, HammingDistance]),
        (type_args.0[1], [L1Sensitivity<u32>, L2Sensitivity<u32>]),
        (type_args.0[2], @primitives)
    ], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "make_identity", "ret": "FfiTransformation *" },
    { "name": "make_split_lines", "args": [ ["const char *", "selector"] ], "ret": "FfiTransformation *" },
    { "name": "make_parse_series", "args": [ ["const char *", "selector"], ["bool", "impute"] ], "ret": "FfiTransformation *" },
    { "name": "make_split_records", "args": [ ["const char *", "selector"], ["const char *", "separator"] ], "ret": "FfiTransformation *" },
    { "name": "make_create_dataframe", "args": [ ["const char *", "selector"], ["unsigned int", "col_count"] ], "ret": "FfiTransformation *" },
    { "name": "make_split_dataframe", "args": [ ["const char *", "selector"], ["const char *", "separator"], ["unsigned int", "col_count"] ], "ret": "FfiTransformation *" },
    { "name": "make_parse_column", "args": [ ["const char *", "selector"], ["void *", "key"], ["bool", "impute"] ], "ret": "FfiTransformation *" },
    { "name": "make_select_column", "args": [ ["const char *", "selector"], ["void *", "key"] ], "ret": "FfiTransformation *" },
    { "name": "make_clamp_vec", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "FfiTransformation *" },
    { "name": "make_clamp_scalar", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "FfiTransformation *" },
    { "name": "make_cast_vec", "args": [ ["const char *", "selector"] ], "ret": "FfiTransformation *" },
    { "name": "make_bounded_sum", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "FfiTransformation *" },
    { "name": "make_count", "args": [ ["const char *", "selector"] ], "ret": "FfiTransformation *" }
]
}"#;
    util::bootstrap(spec)
}
