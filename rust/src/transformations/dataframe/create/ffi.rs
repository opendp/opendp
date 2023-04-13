use std::{os::raw::c_char};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{
        any::{AnyObject, AnyTransformation, Downcast},
        util::{self},
    },
    transformations::{make_create_dataframe, make_split_dataframe},
};

use super::{make_split_lines, make_split_records};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_split_lines() -> FfiResult<*mut AnyTransformation> {
    make_split_lines().into_any()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_split_records(
    separator: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let separator = try_!(util::to_option_str(separator));
    make_split_records(separator).into_any()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_create_dataframe(
    col_names: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<String>>()).clone();
    make_create_dataframe(col_names).into_any()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_split_dataframe(
    separator: *const c_char,
    col_names: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let separator = try_!(util::to_option_str(separator));
    let col_names = try_!(try_as_ref!(col_names).downcast_ref::<Vec<String>>()).clone();
    make_split_dataframe(separator, col_names).into_any()
}

#[cfg(test)]
mod tests {
    use std::ptr::null;

    use polars::df;
    use polars::prelude::*;

    use crate::error::Fallible;

    use crate::core;
    use crate::ffi::any::{AnyObject, Downcast};

    use super::*;

    fn to_owned(strs: &[&'static str]) -> Vec<String> {
        strs.into_iter().map(|s| s.to_owned().to_owned()).collect()
    }

    #[test]
    fn test_make_split_dataframe() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_split_dataframe(
            null(),
            AnyObject::new_raw(vec!["A".to_owned(), "B".to_owned()]),
        ))?;
        let arg = AnyObject::new_raw("1, 1.0\n2, 2.0\n3, 3.0".to_owned());
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: LazyFrame = Fallible::from(res)?.downcast()?;
        assert_eq!(
            res.collect()?,
            df![
                "A" => &["1", "2", "3"],
                "B" => &["1.0", "2.0", "3.0"]
            ]?
        );
        Ok(())
    }
}
