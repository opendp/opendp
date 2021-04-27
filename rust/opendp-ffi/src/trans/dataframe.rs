use std::fmt::Debug;
use std::hash::Hash;
use std::os::raw::{c_char, c_void};
use std::str::FromStr;

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::err;
use opendp::trans::{make_create_data_frame, make_parse_column, make_parse_series, make_select_column, make_split_dataframe, make_split_lines, make_split_records};

use crate::core::{FfiObject, FfiResult, FfiTransformation};
use crate::util::{c_bool, parse_type_args};
use crate::util;

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_lines(type_args: *const c_char) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M>() -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        make_split_lines::<M>().into()
    }
    let type_args = try_!(parse_type_args(type_args, 1));
    dispatch!(monomorphize, [(type_args[0], @dist_dataset)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_series(type_args: *const c_char, impute: c_bool) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, T>(impute: bool) -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone,
              T: 'static + FromStr + Default,
              T::Err: Debug {
        make_parse_series::<M, T>(impute).into()
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args[0], @dist_dataset), (type_args[1], @primitives)], (impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_records(type_args: *const c_char, separator: *const c_char) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M>(separator: Option<&str>) -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        make_split_records::<M>(separator).into()
    }
    let type_args = try_!(parse_type_args(type_args, 1));
    let separator = try_!(util::to_option_str(separator));
    dispatch!(monomorphize, [(type_args[0], @dist_dataset)], (separator))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_create_dataframe(type_args: *const c_char, col_names: *const FfiObject) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, K>(col_names: *const FfiObject) -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone,
              K: 'static + Eq + Hash + Debug + Clone {
        let col_names = try_as_ref!(col_names).as_ref::<Vec<K>>().clone();
        make_create_data_frame::<M, K>(col_names).into()
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    dispatch!(monomorphize, [(type_args[0], @dist_dataset), (type_args[1], @hashable)], (col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_dataframe(type_args: *const c_char, separator: *const c_char, col_names: *const FfiObject) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, K>(separator: Option<&str>, col_names: *const FfiObject) -> FfiResult<*mut FfiTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone,
              K: 'static + Eq + Hash + Debug + Clone {
        let col_names = try_as_ref!(col_names).as_ref::<Vec<K>>().clone();
        make_split_dataframe::<M, K>(separator, col_names).into()
    }
    let type_args = try_!(parse_type_args(type_args, 2));
    let separator = try_!(util::to_option_str(separator));
    dispatch!(monomorphize, [(type_args[0], @dist_dataset), (type_args[1], @hashable)], (separator, col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_column(type_args: *const c_char, key: *const c_void, impute: c_bool) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, K, T>(key: *const c_void, impute: bool) -> FfiResult<*mut FfiTransformation> where
        M: 'static + DatasetMetric<Distance=u32> + Clone,
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq + FromStr + Default,
        T::Err: Debug {
        let key = try_as_ref!(key as *const K).clone();
        make_parse_column::<M, K, T>(key, impute).into()
    }
    let type_args = try_!(parse_type_args(type_args, 3));
    let impute = util::to_bool(impute);
    dispatch!(monomorphize, [(type_args[0], @dist_dataset), (type_args[1], @hashable), (type_args[2], @primitives)], (key, impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(type_args: *const c_char, key: *const c_void) -> FfiResult<*mut FfiTransformation> {
    fn monomorphize<M, K, T>(key: *const c_void) -> FfiResult<*mut FfiTransformation> where
        M: 'static + DatasetMetric<Distance=u32> + Clone,
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq {
        let key = try_as_ref!(key as *const K).clone();
        make_select_column::<M, K, T>(key).into()
    }
    let type_args = try_!(parse_type_args(type_args, 3));
    dispatch!(monomorphize, [(type_args[0], @dist_dataset), (type_args[1], @hashable), (type_args[2], @primitives)], (key))
}
