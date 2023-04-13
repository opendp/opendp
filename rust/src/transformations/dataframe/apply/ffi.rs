use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::domains::{AtomDomain, VectorDomain, LazyFrameDomain};
use crate::err;
use crate::transformations::{
    make_df_cast_default, make_df_is_equal, DatasetMetric,
};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, self};
use crate::traits::{Primitive, RoundCast};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_df_cast_default(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    column_name: *const c_char,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TOA, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        column_name: *const c_char,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: Primitive,
        TOA: Primitive + RoundCast<TIA>,
        M: 'static + DatasetMetric,
        (LazyFrameDomain, M): MetricSpace,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
    {
        let input_domain = try_!(input_domain.downcast_ref::<LazyFrameDomain>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<M>()).clone();
        let column_name = try_!(util::to_str(column_name));
        make_df_cast_default::<TIA, TOA, M>(input_domain, input_metric, column_name).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));
    let M = input_metric.type_.clone();

    dispatch!(monomorphize, [
        (TIA, @primitives),
        (TOA, @primitives),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, column_name))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_df_is_equal(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    column_name: *const c_char,
    value: *const AnyObject,
    TIA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let column_name = try_!(util::to_str(column_name));
    let value = try_as_ref!(value);

    fn monomorphize<TIA, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        column_name: &str,
        value: &AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: Primitive,
        M: 'static + DatasetMetric,
        (LazyFrameDomain, M): MetricSpace,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
        (VectorDomain<AtomDomain<bool>>, M): MetricSpace,
    {
        let input_domain = try_!(input_domain.downcast_ref::<LazyFrameDomain>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<M>()).clone();
        let value: TIA = try_!(value.downcast_ref::<TIA>()).clone();
        make_df_is_equal::<TIA, M>(input_domain, input_metric, column_name, value).into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let M = input_metric.type_.clone();

    dispatch!(monomorphize, [
        (TIA, @primitives),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, column_name, value))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use polars::prelude::*;

    use crate::data::Column;
    use crate::domains::SeriesDomain;
    use crate::error::{ExplainUnwrap, Fallible};
    use crate::metrics::SymmetricDistance;

    use crate::core;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util::ToCharP;

    use super::*;

    fn to_owned(strs: &[&'static str]) -> Vec<String> {
        strs.into_iter().map(|s| s.to_owned().to_owned()).collect()
    }

    #[test]
    fn test_df_cast_default() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_df_cast_default(
            AnyDomain::new_raw(LazyFrameDomain::new(vec![SeriesDomain::new::<String>("A")])?),
            AnyMetric::new_raw(SymmetricDistance::default()),
            "A".to_char_p(),
            "String".to_char_p(),
            "bool".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(df!["A" => &["1", "", "1"]]?.lazy());
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: LazyFrame = Fallible::from(res)?.downcast()?;

        let subset = res.collect()?;

        assert_eq!(subset, df!["A" => &[true, false, true]]?);
        Ok(())
    }

    #[test]
    fn test_df_is_equal() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_df_is_equal(
            AnyDomain::new_raw(LazyFrameDomain::new(vec![SeriesDomain::new::<String>("A")])?),
            AnyMetric::new_raw(SymmetricDistance::default()),
            "A".to_char_p(),
            AnyObject::new_raw("yes".to_string()),
            "String".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(df!["A" => &["yes", "no", "yes"]]?.lazy());
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: HashMap<String, Column> = Fallible::from(res)?.downcast()?;

        let subset = res.get("A").unwrap_test().as_form::<Vec<bool>>()?.clone();

        assert_eq!(subset, vec![true, false, true]);
        Ok(())
    }
}
