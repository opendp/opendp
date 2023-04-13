use std::os::raw::c_char;

use crate::domains::LazyFrameDomain;
use crate::err;
use crate::transformations::{make_subset_by, DatasetMetric};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::ffi::any::{AnyTransformation, Downcast, AnyDomain, AnyMetric};
use crate::ffi::util;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_subset_by(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    indicator_column: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        indicator_column: &str,
    ) -> FfiResult<*mut AnyTransformation>
    where
        M: DatasetMetric,
    {
        let input_domain: LazyFrameDomain =
            try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
        let input_metric: M =
            try_!(try_as_ref!(input_metric).downcast_ref::<M>()).clone();
        make_subset_by::<M>(input_domain, input_metric, indicator_column).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let indicator_column = try_!(util::to_str(indicator_column));
    let M = input_metric.type_;

    dispatch!(monomorphize, [
        (M, @dataset_metrics)
    ], (input_domain, input_metric, indicator_column))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::data::Column;
    use crate::domains::SeriesDomain;
    use crate::error::{ExplainUnwrap, Fallible};

    use crate::core;
    use crate::ffi::any::{AnyObject, Downcast};

    use crate::ffi::util::ToCharP;
    use crate::metrics::SymmetricDistance;

    use polars::prelude::*;
    use super::*;

    fn to_owned(strs: &[&'static str]) -> Vec<String> {
        strs.into_iter().map(|s| s.to_owned().to_owned()).collect()
    }

    #[test]
    fn test_make_subset_by_ffi() -> Fallible<()> {
        let transformation = Result::from(opendp_transformations__make_subset_by(
            AnyDomain::new_raw(LazyFrameDomain::new(vec![
                SeriesDomain::new::<bool>("A"),
                SeriesDomain::new::<String>("B"),
            ])?),
            AnyMetric::new_raw(SymmetricDistance::default()),
            "A".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(df![
            "A" => [true, false, false],
            "B" => ["1.0", "2.0", "3.0"]
        ]?);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: HashMap<String, Column> = Fallible::from(res)?.downcast()?;

        let subset = res.get("B").unwrap_test().as_form::<Vec<String>>()?.clone();

        assert_eq!(subset, vec!["1.0".to_string()]);
        Ok(())
    }
}
