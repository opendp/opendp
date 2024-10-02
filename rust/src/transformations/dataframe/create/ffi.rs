use std::{convert::TryFrom, os::raw::c_char};

#[allow(deprecated)]
use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    error::Fallible,
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::{self, Type},
    },
    traits::Hashable,
    transformations::{make_create_dataframe, make_split_dataframe},
};

use super::{make_split_lines, make_split_records};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_split_lines() -> FfiResult<*mut AnyTransformation> {
    make_split_lines().into_any().into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_split_records(
    separator: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let separator = try_!(util::to_option_str(separator));
    make_split_records(separator).into_any().into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_create_dataframe(
    col_names: *const AnyObject,
    K: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K>(col_names: *const AnyObject) -> Fallible<AnyTransformation>
    where
        K: Hashable,
    {
        let col_names = try_as_ref!(col_names).downcast_ref::<Vec<K>>()?.clone();
        #[allow(deprecated)]
        make_create_dataframe::<K>(col_names).into_any()
    }
    let K = try_!(Type::try_from(K));
    dispatch!(monomorphize, [(K, @hashable)], (col_names)).into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_split_dataframe(
    separator: *const c_char,
    col_names: *const AnyObject,
    K: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<K>(
        separator: Option<&str>,
        col_names: *const AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        K: Hashable,
    {
        let col_names = try_as_ref!(col_names).downcast_ref::<Vec<K>>()?.clone();
        #[allow(deprecated)]
        make_split_dataframe::<K>(separator, col_names).into_any()
    }
    let K = try_!(Type::try_from(K));
    let separator = try_!(util::to_option_str(separator));

    dispatch!(monomorphize, [(K, @hashable)], (separator, col_names)).into()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ptr::null;

    use crate::data::Column;
    use crate::error::Fallible;
    use crate::transformations::DataFrame;

    use crate::core;
    use crate::ffi::any::{AnyObject, Downcast};
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
        let transformation = Result::from(opendp_transformations__make_split_dataframe(
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
