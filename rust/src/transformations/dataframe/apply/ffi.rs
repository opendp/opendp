use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::err;
use crate::transformations::{make_df_cast_default, make_df_is_equal};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{Hashable, Primitive, RoundCast};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_df_cast_default(
    column_name: *const AnyObject,
    TK: *const c_char,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK, TIA, TOA>(
        column_name: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TK: Hashable,
        TIA: Primitive,
        TOA: Primitive + RoundCast<TIA>,
    {
        let column_name: TK = try_!(try_as_ref!(column_name).downcast_ref::<TK>()).clone();
        make_df_cast_default::<TK, TIA, TOA>(column_name).into_any()
    }
    let TK = try_!(Type::try_from(TK));
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TIA, @primitives),
        (TOA, @primitives)
    ], (column_name))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_df_is_equal(
    column_name: *const AnyObject,
    value: *const AnyObject,
    TK: *const c_char,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK, TIA>(
        column_name: *const AnyObject,
        value: *const AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TK: Hashable,
        TIA: Primitive,
    {
        let column_name: TK = try_!(try_as_ref!(column_name).downcast_ref::<TK>()).clone();
        let value: TIA = try_!(try_as_ref!(value).downcast_ref::<TIA>()).clone();
        make_df_is_equal::<TK, TIA>(column_name, value).into_any()
    }
    let TK = try_!(Type::try_from(TK));
    let TIA = try_!(Type::try_from(TIA));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TIA, @primitives)
    ], (column_name, value))
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
    fn test_df_cast_default() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_df_cast_default(
            AnyObject::new_raw("A".to_string()),
            "String".to_char_p(),
            "String".to_char_p(),
            "bool".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(dataframe(vec![(
            "A",
            Column::new(to_owned(&["1", "", "1"])),
        )]));
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: HashMap<String, Column> = Fallible::from(res)?.downcast()?;

        let subset = res.get("A").unwrap_test().as_form::<Vec<bool>>()?.clone();

        assert_eq!(subset, vec![true, false, true]);
        Ok(())
    }

    #[test]
    fn test_df_is_equal() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_df_is_equal(
            AnyObject::new_raw("A".to_string()),
            AnyObject::new_raw("yes".to_string()),
            "String".to_char_p(),
            "String".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(dataframe(vec![(
            "A",
            Column::new(to_owned(&["yes", "no", "yes"])),
        )]));
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: HashMap<String, Column> = Fallible::from(res)?.downcast()?;

        let subset = res.get("A").unwrap_test().as_form::<Vec<bool>>()?.clone();

        assert_eq!(subset, vec![true, false, true]);
        Ok(())
    }
}
