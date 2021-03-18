use std::fmt::Debug;
use std::iter::Sum;
use std::os::raw::{c_char, c_uint, c_void};
use std::str::FromStr;

use opendp::trans::{MakeTransformation0, MakeTransformation1, MakeTransformation2};
use opendp::dist::{HammingDistance, SymmetricDistance, L1Sensitivity, L2Sensitivity};
use opendp::dom::AllDomain;
use opendp::trans;

use crate::core::FfiTransformation;
use crate::util;
use crate::util::{c_bool};
use crate::util::TypeArgs;
use std::ops::{Sub, Mul, Div};
use opendp::traits::DistanceCast;
use std::hash::Hash;
use opendp::core::{DatasetMetric, SensitivityMetric};
use num::Signed;

// TODO: dispatch based on Metric type

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(type_args: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<T: 'static + Clone>() -> *mut FfiTransformation {
        let transformation = trans::Identity::make(AllDomain::<T>::new(), HammingDistance::new()).unwrap();
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
    fn monomorphize<T, M>(impute: bool) -> *mut FfiTransformation
        where T: 'static + FromStr + Default,
              T::Err: Debug,
              M: 'static + DatasetMetric<Distance=u32> + Clone {
        let transformation = trans::ParseSeries::<T, M>::make(impute).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args.0[0], @primitives), (type_args.0[1], @dist_dataset)], (impute))
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
    dispatch!(monomorphize, [(type_args.0[1], @dist_dataset)], (separator))
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
    fn monomorphize<K, T, M>(key: *const c_void, impute: bool) -> *mut FfiTransformation where
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq + FromStr + Default,
        T::Err: Debug,
        M: 'static + DatasetMetric<Distance=u32> + Clone {
        let key = util::as_ref(key as *const K).clone();
        let transformation = trans::ParseColumn::<M, T>::make(key, impute).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 3);
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args.0[0], @hashable), (type_args.0[1], @primitives), (type_args.0[2], @dist_dataset)], (key, impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(type_args: *const c_char, key: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<K, T, M>(key: *const c_void) -> *mut FfiTransformation where
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq,
        M: 'static + DatasetMetric<Distance=u32> + Clone {
        let key = util::as_ref(key as *const K).clone();
        let transformation = trans::SelectColumn::<M, T>::make(key).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [(type_args.0[0], @hashable), (type_args.0[1], @primitives), (type_args.0[2], @dist_dataset)], (key))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<T, M>(lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation where
        T: 'static + Copy + PartialOrd,
        M: 'static + DatasetMetric<Distance=u32> + Clone {
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();
        let transformation = trans::Clamp::<M, T>::make(lower, upper).unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 2);
    dispatch!(monomorphize, [(type_args.0[0], @numbers), (type_args.0[1], @dist_dataset)], (lower, upper))
}

// #[no_mangle]
// pub extern "C" fn opendp_trans__make_bounded_sum_borked(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
//
//     fn monomorphize<T>(type_args: TypeArgs, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation
//         where T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast + Signed {
//         fn monomorphize2<T2, MO>(lower: T2, upper: T2) -> *mut FfiTransformation
//             where T2: 'static + Copy + PartialOrd + Sub<Output=T2> + Mul<Output=T2> + Div<Output=T2> + Sum<T2> + DistanceCast + Signed,
//                   MI: 'static + DatasetMetric<Distance=u32> + Clone,
//                   MO: 'static + SensitivityMetric<Distance=T2> + Clone {
//             let transformation = trans::BoundedSum::<MI, MO, T2>::make2(lower, upper).unwrap();
//             FfiTransformation::new_from_types(transformation)
//         }
//         let lower = util::as_ref(lower as *const T).clone();
//         let upper = util::as_ref(upper as *const T).clone();
//         dispatch!(monomorphize2, [(type_args.0[0], [T]), (type_args.0[1], [SymmetricDistance, HammingDistance]), (type_args.0[2], [L1Sensitivity<T>, L2Sensitivity<T>])], (lower, upper))
//     }
//     let type_args = TypeArgs::expect(type_args, 3);
//     dispatch!(monomorphize, [(type_args.0[0], @numbers)], (type_args, lower, upper))
// }

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {

    fn monomorphize<T>(type_args: TypeArgs, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation
        where T: 'static + Copy + PartialOrd + Sub<Output=T> + Mul<Output=T> + Div<Output=T> + Sum<T> + DistanceCast + Signed {

        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();

        match (type_args.0[1]).id {
            x   if x == std::any::TypeId::of::<SymmetricDistance>() => match (type_args.0[2]).id {
                x   if x == std::any::TypeId::of::<L1Sensitivity<T>>() =>
                    FfiTransformation::new_from_types(trans::BoundedSum::<SymmetricDistance, L1Sensitivity<T>, T>::make2(lower, upper).unwrap()),
                x   if x == std::any::TypeId::of::<L2Sensitivity<T>>() =>
                    FfiTransformation::new_from_types(trans::BoundedSum::<SymmetricDistance, L2Sensitivity<T>, T>::make2(lower, upper).unwrap()),
                _ => unreachable!()
            },
            x   if x == std::any::TypeId::of::<HammingDistance>() => match (type_args.0[2]).id {
                x   if x == std::any::TypeId::of::<L1Sensitivity<T>>() =>
                    FfiTransformation::new_from_types(trans::BoundedSum::<HammingDistance, L1Sensitivity<T>, T>::make2(lower, upper).unwrap()),
                x   if x == std::any::TypeId::of::<L2Sensitivity<T>>() =>
                    FfiTransformation::new_from_types(trans::BoundedSum::<HammingDistance, L2Sensitivity<T>, T>::make2(lower, upper).unwrap()),
                _ => unreachable!()
            },
            _ => unreachable!()
        }
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [(type_args.0[0], @signed_numbers)], (type_args, lower, upper))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_count(type_args: *const c_char) -> *mut FfiTransformation {

    fn monomorphize<T: 'static, MI, MO>() -> *mut FfiTransformation
        where MI: 'static + DatasetMetric<Distance=u32> + Clone,
              MO: 'static + SensitivityMetric<Distance=u32> + Clone {
        let transformation = trans::Count::<MI, MO, T>::make().unwrap();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 3);
    dispatch!(monomorphize, [(type_args.0[0], @primitives), (type_args.0[1], [SymmetricDistance, HammingDistance]), (type_args.0[2], [L1Sensitivity<u32>, L2Sensitivity<u32>])], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "make_identity", "ret": "void *" },
    { "name": "make_split_lines", "args": [ ["const char *", "selector"] ], "ret": "void *" },
    { "name": "make_parse_series", "args": [ ["const char *", "selector"], ["bool", "impute"] ], "ret": "void *" },
    { "name": "make_split_records", "args": [ ["const char *", "selector"], ["const char *", "separator"] ], "ret": "void *" },
    { "name": "make_create_dataframe", "args": [ ["const char *", "selector"], ["unsigned int", "col_count"] ], "ret": "void *" },
    { "name": "make_split_dataframe", "args": [ ["const char *", "selector"], ["const char *", "separator"], ["unsigned int", "col_count"] ], "ret": "void *" },
    { "name": "make_parse_column", "args": [ ["const char *", "selector"], ["void *", "key"], ["bool", "impute"] ], "ret": "void *" },
    { "name": "make_select_column", "args": [ ["const char *", "selector"], ["void *", "key"] ], "ret": "void *" },
    { "name": "make_clamp", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
    { "name": "make_bounded_sum", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
    { "name": "make_count", "args": [ ["const char *", "selector"] ], "ret": "void *" }
]
}"#;
    util::bootstrap(spec)
}
