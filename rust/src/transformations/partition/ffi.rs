use std::{convert::TryFrom, os::raw::c_char};

use crate::{
    core::{FfiResult, Function, IntoAnyTransformationFfiResultExt, Transformation, StabilityMap},
    domains::ProductDomain,
    ffi::{
        any::{AnyDomain, AnyObject, AnyTransformation, Downcast, AnyMetric},
        util::{Type, c_bool, to_bool},
    },
    traits::Hashable,
    transformations::{make_partition_by, DataFrame}, 
    metrics::{ProductMetric, IntDistance},
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_partition_by(
    identifier_column: *const AnyObject,
    partition_keys: *const AnyObject,
    keep_columns: *const AnyObject,
    null_partition: c_bool,
    TK: *const c_char,
    TV: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK: Hashable, TV: Hashable>(
        identifier_column: *const AnyObject,
        partition_keys: *const AnyObject,
        keep_columns: *const AnyObject,
        null_partition: bool
    ) -> FfiResult<*mut AnyTransformation> {
        let identifier_column = try_!(try_as_ref!(identifier_column).downcast_ref::<TK>()).clone();
        let partition_keys = try_!(try_as_ref!(partition_keys).downcast_ref::<Vec<TV>>()).clone();
        let keep_columns = try_!(try_as_ref!(keep_columns).downcast_ref::<Vec<TK>>()).clone();
        let trans = try_!(make_partition_by::<TK, TV>(
            identifier_column,
            partition_keys,
            keep_columns,
            null_partition
        ));

        // rewrite the partitioner to emit ProductDomain<AnyDomain>, and box output partitions in the function
        let inner_output_domains = (trans.output_domain.inner_domains)
            .into_iter()
            .map(AnyDomain::new)
            .collect();
        let function = trans.function;
        let stability_map = trans.stability_map;
        Ok(Transformation::new(
            trans.input_domain,
            ProductDomain::new(inner_output_domains),
            Function::new_fallible(move |arg: &DataFrame<TK>| {
                let res = function.eval(arg);
                res.map(|o| {
                    o.into_iter()
                        .map(AnyObject::new)
                        .collect::<Vec<AnyObject>>()
                })
            }),
            trans.input_metric,
            ProductMetric::new(AnyMetric::new(trans.output_metric.inner_metric)),
            StabilityMap::new_fallible(move |d_in: &IntDistance| stability_map.eval(d_in).map(AnyObject::new))
        ))
        .into_any()
    }

    let null_partition = to_bool(null_partition);
    let TK = try_!(Type::try_from(TK));
    let TV = try_!(Type::try_from(TV));
    dispatch!(monomorphize, [
        (TK, @hashable),
        (TV, @hashable)
    ], (identifier_column, partition_keys, keep_columns, null_partition))
}
