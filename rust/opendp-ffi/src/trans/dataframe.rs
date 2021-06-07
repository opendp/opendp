use std::convert::TryFrom;
use std::fmt::Debug;
use std::hash::Hash;
use std::os::raw::{c_char, c_void};
use std::str::FromStr;

use opendp::core::DatasetMetric;
use opendp::dist::{HammingDistance, SymmetricDistance};
use opendp::err;
use opendp::trans::{make_create_dataframe, make_parse_column, make_parse_series, make_select_column, make_split_dataframe, make_split_lines, make_split_records};

use crate::any::{AnyObject, AnyTransformation, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{c_bool, Type};
use crate::util;

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_lines(
    M: *const c_char
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M>() -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        make_split_lines::<M>().into_any()
    }
    let M = try_!(Type::try_from(M));
    dispatch!(monomorphize, [(M, @dist_dataset)], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_series(
    impute: c_bool,
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, T>(impute: bool) -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone,
              T: 'static + FromStr + Default,
              T::Err: Debug {
        make_parse_series::<M, T>(impute).into_any()
    }
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));
    let impute = util::to_bool(impute);

    dispatch!(monomorphize, [
        (M, @dist_dataset),
        (T, @primitives)
    ], (impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_records(
    separator: *const c_char,
    M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M>(separator: Option<&str>) -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone {
        make_split_records::<M>(separator).into_any()
    }
    let M = try_!(Type::try_from(M));
    let separator = try_!(util::to_option_str(separator));

    dispatch!(monomorphize, [(M, @dist_dataset)], (separator))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_create_dataframe(
    col_names: *const AnyObject,
    M: *const c_char, K: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, K>(col_names: *const AnyObject) -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone,
              K: 'static + Eq + Hash + Debug + Clone {
        let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<K>>()).clone();
        make_create_dataframe::<M, K>(col_names).into_any()
    }
    let M = try_!(Type::try_from(M));
    let K = try_!(Type::try_from(K));
    dispatch!(monomorphize, [
        (M, @dist_dataset),
        (K, @hashable)
    ], (col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_dataframe(
    separator: *const c_char, col_names: *const AnyObject,
    M: *const c_char, K: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, K>(separator: Option<&str>, col_names: *const AnyObject) -> FfiResult<*mut AnyTransformation>
        where M: 'static + DatasetMetric<Distance=u32> + Clone,
              K: 'static + Eq + Hash + Debug + Clone {
        let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<K>>()).clone();
        make_split_dataframe::<M, K>(separator, col_names).into_any()
    }
    let M = try_!(Type::try_from(M));
    let K = try_!(Type::try_from(K));
    let separator = try_!(util::to_option_str(separator));

    dispatch!(monomorphize, [(M, @dist_dataset), (K, @hashable)], (separator, col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_column(
    key: *const c_void, impute: c_bool,
    M: *const c_char, K: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, K, T>(key: *const c_void, impute: bool) -> FfiResult<*mut AnyTransformation> where
        M: 'static + DatasetMetric<Distance=u32> + Clone,
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq + FromStr + Default,
        T::Err: Debug {
        let key = try_as_ref!(key as *const K).clone();
        make_parse_column::<M, K, T>(key, impute).into_any()
    }
    let M = try_!(Type::try_from(M));
    let K = try_!(Type::try_from(K));
    let T = try_!(Type::try_from(T));
    let impute = util::to_bool(impute);

    dispatch!(monomorphize, [
        (M, @dist_dataset),
        (K, @hashable),
        (T, @primitives)
    ], (key, impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(
    key: *const c_void,
    M: *const c_char, K: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M, K, T>(key: *const c_void) -> FfiResult<*mut AnyTransformation> where
        M: 'static + DatasetMetric<Distance=u32> + Clone,
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq {
        let key = try_as_ref!(key as *const K).clone();
        make_select_column::<M, K, T>(key).into_any()
    }
    let M = try_!(Type::try_from(M));
    let K = try_!(Type::try_from(K));
    let T = try_!(Type::try_from(T));

    dispatch!(monomorphize, [
        (M, @dist_dataset),
        (K, @hashable),
        (T, @primitives)
    ], (key))
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ptr::null;

    use opendp::data::Column;
    use opendp::error::Fallible;
    use opendp::trans::DataFrame;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util;
    use crate::util::ToCharP;

    use super::*;

    fn to_owned(strs: &[&'static str]) -> Vec<String> {
        strs.into_iter().map(|s| s.to_owned().to_owned()).collect()
    }

    fn dataframe(pairs: Vec<(&str, Column)>) -> DataFrame<String> {
        pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
    }

    #[test]
    fn test_make_split_dataframe() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_split_dataframe(
            null(),
            AnyObject::new_raw(vec!["A".to_owned(), "B".to_owned()]),
            "SymmetricDistance".to_char_p(),
            "String".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw("1, 1.0\n2, 2.0\n3, 3.0".to_owned());
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: HashMap<String, Column> = Fallible::from(res)?.downcast()?;
        assert_eq!(
            res,
            dataframe(vec![
                ("A", Column::new(to_owned(&["1", "2", "3"]))),
                ("B", Column::new(to_owned(&["1.0", "2.0", "3.0"])))
            ])
        );
        Ok(())
    }

    #[test]
    fn test_make_parse_column() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_parse_column(
            util::into_raw("A".to_owned()) as *const c_void,
            util::from_bool(true),
            "SymmetricDistance".to_char_p(),
            "String".to_char_p(),
            "i32".to_char_p(),
        ))?;
        let arg = util::into_raw(AnyObject::new(dataframe(vec![
            ("A", Column::new(to_owned(&["1", "2", "3"]))),
            ("B", Column::new(to_owned(&["1.0", "2.0", "3.0"])))
        ])));
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: HashMap<String, Column> = Fallible::from(res)?.downcast()?;
        assert_eq!(
            res,
            dataframe(vec![
                ("A", Column::new(vec![1, 2, 3])),
                ("B", Column::new(to_owned(&["1.0", "2.0", "3.0"])))
            ])
        );
        Ok(())
    }
}
