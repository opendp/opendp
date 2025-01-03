use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::error::Fallible;
#[allow(deprecated)]
use crate::transformations::make_subset_by;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::Hashable;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_subset_by(
    indicator_column: *const AnyObject,
    keep_columns: *const AnyObject,
    TK: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK>(
        indicator_column: *const AnyObject,
        keep_columns: *const AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TK: Hashable,
    {
        let indicator_column: TK = try_as_ref!(indicator_column).downcast_ref::<TK>()?.clone();
        let keep_columns: Vec<TK> = try_as_ref!(keep_columns).downcast_ref::<Vec<TK>>()?.clone();
        #[allow(deprecated)]
        make_subset_by::<TK>(indicator_column, keep_columns).into_any()
    }
    let TK = try_!(Type::try_from(TK));

    dispatch!(monomorphize, [
        (TK, @hashable)
    ], (indicator_column, keep_columns))
    .into()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::data::Column;
    use crate::error::{ExplainUnwrap, Fallible};
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
    fn test_make_subset_by_ffi() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_subset_by(
            AnyObject::new_raw("A".to_string()),
            AnyObject::new_raw(vec!["B".to_owned()]),
            "String".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(dataframe(vec![
            ("A", Column::new(vec![true, false, false])),
            ("B", Column::new(to_owned(&["1.0", "2.0", "3.0"]))),
        ]));
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: HashMap<String, Column> = Fallible::from(res)?.downcast()?;

        let subset = res.get("B").unwrap_test().as_form::<Vec<String>>()?.clone();

        assert_eq!(subset, vec!["1.0".to_string()]);
        Ok(())
    }
}
