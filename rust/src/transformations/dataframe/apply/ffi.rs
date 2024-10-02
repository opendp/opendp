use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::domains::{AtomDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
#[allow(deprecated)]
use crate::transformations::{
    make_df_cast_default, make_df_is_equal, DataFrameDomain, DatasetMetric,
};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{Hashable, Primitive, RoundCast};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_df_cast_default(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    column_name: *const AnyObject,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK, TIA, TOA, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        column_name: *const AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TK: Hashable,
        TIA: Primitive,
        TOA: Primitive + RoundCast<TIA>,
        M: 'static + DatasetMetric,
        (DataFrameDomain<TK>, M): MetricSpace,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
    {
        let input_domain = input_domain.downcast_ref::<DataFrameDomain<TK>>()?.clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let column_name: TK = try_as_ref!(column_name).downcast_ref::<TK>()?.clone();
        #[allow(deprecated)]
        make_df_cast_default::<TK, TIA, TOA, M>(input_domain, input_metric, column_name).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TK = try_!(input_domain.type_.get_atom());
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));
    let M = input_metric.type_.clone();

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TIA, @primitives),
        (TOA, @primitives),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, column_name))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_df_is_equal(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    column_name: *const AnyObject,
    value: *const AnyObject,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let column_name = try_as_ref!(column_name);
    let value = try_as_ref!(value);

    fn monomorphize<TK, TIA, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        column_name: &AnyObject,
        value: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TK: Hashable,
        TIA: Primitive,
        M: 'static + DatasetMetric,
        (DataFrameDomain<TK>, M): MetricSpace,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
    {
        let input_domain = input_domain.downcast_ref::<DataFrameDomain<TK>>()?.clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let column_name: TK = column_name.downcast_ref::<TK>()?.clone();
        let value: TIA = value.downcast_ref::<TIA>()?.clone();
        #[allow(deprecated)]
        make_df_is_equal::<TK, TIA, M>(input_domain, input_metric, column_name, value).into_any()
    }
    let TK = try_!(input_domain.type_.get_atom());
    let TIA = try_!(Type::try_from(TIA));
    let M = input_metric.type_.clone();

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TIA, @primitives),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, column_name, value))
    .into()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::data::Column;
    use crate::error::{ExplainUnwrap, Fallible};
    use crate::metrics::SymmetricDistance;
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
            AnyDomain::new_raw(DataFrameDomain::<String>::new()),
            AnyMetric::new_raw(SymmetricDistance::default()),
            AnyObject::new_raw("A".to_string()),
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
            AnyDomain::new_raw(DataFrameDomain::<String>::new()),
            AnyMetric::new_raw(SymmetricDistance::default()),
            AnyObject::new_raw("A".to_string()),
            AnyObject::new_raw("yes".to_string()),
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
