use std::convert::TryFrom;
use std::fmt::Debug;
use std::hash::Hash;
use std::os::raw::{c_char, c_void};
use std::str::FromStr;

use opendp::err;
use opendp::trans::{make_create_dataframe, make_parse_column, make_select_column, make_split_dataframe, make_split_lines, make_split_records};

use crate::any::{AnyObject, AnyTransformation, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{c_bool, Type};
use crate::util;

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_lines() -> FfiResult<*mut AnyTransformation> {
    make_split_lines().into_any()
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_records(
    separator: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let separator = try_!(util::to_option_str(separator));
    make_split_records(separator).into_any()
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_create_dataframe(
    col_names: *const AnyObject, K: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K>(col_names: *const AnyObject) -> FfiResult<*mut AnyTransformation>
        where K: 'static + Eq + Hash + Clone {
        let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<K>>()).clone();
        make_create_dataframe::<K>(col_names).into_any()
    }
    let K = try_!(Type::try_from(K));
    dispatch!(monomorphize, [(K, @hashable)], (col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_split_dataframe(
    separator: *const c_char, col_names: *const AnyObject,
    K: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K>(separator: Option<&str>, col_names: *const AnyObject) -> FfiResult<*mut AnyTransformation>
        where K: 'static + Eq + Hash + Debug + Clone {
        let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<K>>()).clone();
        make_split_dataframe::<K>(separator, col_names).into_any()
    }
    let K = try_!(Type::try_from(K));
    let separator = try_!(util::to_option_str(separator));

    dispatch!(monomorphize, [(K, @hashable)], (separator, col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_parse_column(
    key: *const c_void, impute: c_bool,
    K: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K, T>(key: *const c_void, impute: bool) -> FfiResult<*mut AnyTransformation> where
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq + FromStr + Default,
        T::Err: Debug {
        let key = try_as_ref!(key as *const K).clone();
        make_parse_column::<K, T>(key, impute).into_any()
    }
    let K = try_!(Type::try_from(K));
    let T = try_!(Type::try_from(T));
    let impute = util::to_bool(impute);

    dispatch!(monomorphize, [
        (K, @hashable),
        (T, @primitives)
    ], (key, impute))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(
    key: *const c_void, K: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K, T>(key: *const c_void) -> FfiResult<*mut AnyTransformation> where
        K: 'static + Hash + Eq + Debug + Clone,
        T: 'static + Debug + Clone + PartialEq {
        let key = try_as_ref!(key as *const K).clone();
        make_select_column::<K, T>(key).into_any()
    }
    let K = try_!(Type::try_from(K));
    let T = try_!(Type::try_from(T));

    dispatch!(monomorphize, [
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
