use std::{convert::TryFrom, os::raw::c_char};

use crate::{
    core::{FfiResult, Function, IntoAnyTransformationFfiResultExt, StabilityMap, Transformation},
    domains::ProductDomain,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
        util::{c_bool, to_bool, Type},
    },
    metrics::{IntDistance, ProductMetric},
    traits::Hashable,
    transformations::{make_sized_partition_by, DataFrame},
    transformations::dataframe::SizedDataFrameDomain,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_partition_by(
    input_domain: *const AnyObject,
    identifier_column: *const AnyObject,
    keep_columns: *const AnyObject,
    null_partition: c_bool,
    TC: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TC: Hashable>(
        input_domain: *const AnyObject,
        identifier_column: *const AnyObject,
        keep_columns: *const AnyObject,
        null_partition: bool,
    ) -> FfiResult<*mut AnyTransformation> {
        let identifier_column = try_!(try_as_ref!(identifier_column).downcast_ref::<TC>()).clone();
        let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<SizedDataFrameDomain<TC>>()).clone();
        let keep_columns = try_!(try_as_ref!(keep_columns).downcast_ref::<Vec<TC>>()).clone();
        let trans = try_!(make_sized_partition_by::<TC>(
            input_domain,
            identifier_column,
            keep_columns,
            null_partition
        ));

        // rewrite the partitioner to emit ProductDomain<InputDomain>, and box output partitions in the function
        let inner_output_domains = (trans.output_domain.inner_domains)
            .into_iter()
            .map(AnyDomain::new)
            .collect();
        let function = trans.function;
        let stability_map = trans.stability_map;
        Ok(Transformation::new(
            trans.input_domain,
            ProductDomain::new(inner_output_domains),
            Function::new_fallible(move |arg: &DataFrame<TC>| {
                let res = function.eval(arg);
                res.map(|o| {
                    o.into_iter()
                        .map(AnyObject::new)
                        .collect::<Vec<AnyObject>>()
                })
            }),
            trans.input_metric,
            ProductMetric::new(AnyMetric::new(trans.output_metric.inner_metric)),
            StabilityMap::new_fallible(move |d_in: &IntDistance| {
                let (k, r) = stability_map.eval(d_in)?;
                Ok((AnyObject::new(k), r))
            }),
        ))
        .into_any()
    }

    let null_partition = to_bool(null_partition);
    let TC = try_!(Type::try_from(TC));
    dispatch!(monomorphize, [
        (TC, @hashable)
    ], (input_domain, identifier_column, keep_columns, null_partition))
}