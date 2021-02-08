use std::fmt::Debug;
use std::iter::Sum;
use std::os::raw::{c_char, c_uint, c_void};
use std::str::FromStr;

use opendp::data::{Element, Form};
use opendp::trans;

use crate::core::FfiTransformation;
use crate::util;
use crate::util::c_bool;
use crate::util::TypeArgs;

#[no_mangle]
pub extern "C" fn opendp_trans__make_identity(type_args: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<T: 'static + Form + Clone>() -> *mut FfiTransformation {
        let transformation = trans::make_identity::<T>();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_lines() -> *mut FfiTransformation {
    let transformation = trans::make_split_lines();
    FfiTransformation::new_from_types(transformation)
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_series(type_args: *const c_char, impute: c_bool) -> *mut FfiTransformation {
    fn monomorphize<T>(impute: bool) -> *mut FfiTransformation where
        T: 'static + FromStr + Default, T::Err: Debug {
        let transformation = trans::make_parse_series::<T>(impute);
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args.0[0], @primitives)], (impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_records(separator: *const c_char) -> *mut FfiTransformation {
    let separator = util::to_option_str(separator);
    let transformation = trans::make_split_records(separator);
    FfiTransformation::new_from_types(transformation)
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_create_dataframe(col_count: c_uint) -> *mut FfiTransformation {
    let col_count = col_count as usize;
    let transformation = trans::make_create_dataframe(col_count);
    FfiTransformation::new_from_types(transformation)
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_dataframe(separator: *const c_char, col_count: c_uint) -> *mut FfiTransformation {
    let separator = util::to_option_str(separator);
    let col_count = col_count as usize;
    let transformation = trans::make_split_dataframe(separator, col_count);
    FfiTransformation::new_from_types(transformation)
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_column(type_args: *const c_char, key: *const c_char, impute: c_bool) -> *mut FfiTransformation {
    fn monomorphize<T>(key: &str, impute: bool) -> *mut FfiTransformation where
        T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
        let transformation = trans::make_parse_column::<T>(key, impute);
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let key = util::to_str(key);
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args.0[0], @primitives)], (key, impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(type_args: *const c_char, key: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<T>(key: &str) -> *mut FfiTransformation where
        T: 'static + Element + Clone + PartialEq {
        let transformation = trans::make_select_column::<T>(key);
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    let key = util::to_str(key);
    dispatch!(monomorphize, [(type_args.0[0], @primitives)], (key))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<T>(lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation where
        T: 'static + Copy + PartialOrd {
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();
        let transformation = trans::make_clamp::<T>(lower, upper);
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (lower, upper))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum_l1(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<T>(lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation where
        T: 'static + Clone + PartialOrd + Sum {
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();
        let transformation = trans::make_bounded_sum_l1::<T>(lower, upper);
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (lower, upper))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum_l2(type_args: *const c_char, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
    fn monomorphize<T>(lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation where
        T: 'static + Clone + PartialOrd + Sum {
        let lower = util::as_ref(lower as *const T).clone();
        let upper = util::as_ref(upper as *const T).clone();
        let transformation = trans::make_bounded_sum_l2::<T>(lower, upper);
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @numbers)], (lower, upper))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_count_l1(type_args: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<T>() -> *mut FfiTransformation where T: 'static {
        let transformation = trans::make_count_l1::<T>();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_count_l2(type_args: *const c_char) -> *mut FfiTransformation {
    fn monomorphize<T>() -> *mut FfiTransformation where T: 'static {
        let transformation = trans::make_count_l2::<T>();
        FfiTransformation::new_from_types(transformation)
    }
    let type_args = TypeArgs::expect(type_args, 1);
    dispatch!(monomorphize, [(type_args.0[0], @primitives)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "make_identity", "ret": "void *" },
    { "name": "make_split_lines", "ret": "void *" },
    { "name": "make_parse_series", "args": [ ["const char *", "selector"], ["bool", "impute"] ], "ret": "void *" },
    { "name": "make_split_records", "args": [ ["const char *", "separator"] ], "ret": "void *" },
    { "name": "make_create_dataframe", "args": [ ["unsigned int", "col_count"] ], "ret": "void *" },
    { "name": "make_split_dataframe", "args": [ ["const char *", "separator"], ["unsigned int", "col_count"] ], "ret": "void *" },
    { "name": "make_parse_column", "args": [ ["const char *", "selector"], ["const char *", "key"], ["bool", "impute"] ], "ret": "void *" },
    { "name": "make_select_column", "args": [ ["const char *", "selector"], ["const char *", "key"] ], "ret": "void *" },
    { "name": "make_clamp", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
    { "name": "make_bounded_sum_l1", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
    { "name": "make_bounded_sum_l2", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
    { "name": "make_count_l1", "args": [ ["const char *", "selector"] ], "ret": "void *" },
    { "name": "make_count_l2", "args": [ ["const char *", "selector"] ], "ret": "void *" }
]
}"#;
    util::bootstrap(spec)
}
