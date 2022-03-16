use std::convert::TryFrom;
use std::fmt::Debug;
use std::hash::Hash;
use std::os::raw::c_char;

use crate::err;
use crate::trans::{make_create_dataframe, make_select_column, make_split_dataframe, make_split_lines, make_split_records};

use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::util::Type;
use crate::ffi::util;
use crate::traits::CheckNull;

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
        where K: 'static + Eq + Hash + Clone + CheckNull {
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
        where K: 'static + Eq + Hash + Debug + Clone + CheckNull {
        let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<K>>()).clone();
        make_split_dataframe::<K>(separator, col_names).into_any()
    }
    let K = try_!(Type::try_from(K));
    let separator = try_!(util::to_option_str(separator));

    dispatch!(monomorphize, [(K, @hashable)], (separator, col_names))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_select_column(
    key: *const AnyObject, K: *const c_char, TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K, TOA>(key: *const AnyObject) -> FfiResult<*mut AnyTransformation> where
        K: 'static + Hash + Eq + Debug + Clone + CheckNull,
        TOA: 'static + Debug + Clone + PartialEq + CheckNull {
        let key: K = try_!(try_as_ref!(key).downcast_ref::<K>()).clone();
        make_select_column::<K, TOA>(key).into_any()
    }
    let K = try_!(Type::try_from(K));
    let TOA = try_!(Type::try_from(TOA));

    dispatch!(monomorphize, [
        (K, @hashable),
        (TOA, @primitives)
    ], (key))
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ptr::null;

    use crate::data::Column;
    use crate::error::Fallible;
    use crate::trans::DataFrame;

    use crate::ffi::any::{AnyObject, Downcast};
    use crate::core;
    use crate::ffi::util::ToCharP;

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
}
