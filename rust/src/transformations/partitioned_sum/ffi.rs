/* use std::{convert::TryFrom, os::raw::c_char};

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
pub extern "C" fn opendp_transformations_make_sized_partitioned_sum(
    input_domain: *const AnyObject,
    partition_column: *const AnyObject,
    sum_column: *const AnyObject,
    bounds: *const AnyObject,
    null_partition: c_bool,
    T: *const c_????,
) -> FfiResult<*mut AnyTransformation> {
    
    fn monomorphize<T: Float>(
        input_domain: *const AnyObject,
        partition_column: *const AnyObject,
        sum_column: *const AnyObject,
        bounds: *const AnyObject,
        null_partition: bool,
    ) -> FfiResult<*mut AnyTransformation> {

        let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
        let partition_column = try_!(try_as_ref!(identifier_column).downcast_ref::<&str>()).clone();
        let sum_column = try_!(try_as_ref!(identifier_column).downcast_ref::<&str>()).clone();
    
        let trans = try_!(make_sized_partitioned_sum::<T>(
            input_domain,
            partition_column,
            sum_column,
            bounds,
            null_partition
        ));

        // rewrite the partitioner to emit ProductDomain<InputDomain>, and box output partitions in the function
        let inner_output_domain = trans.output_domain;
        let function = trans.function;
        let stability_map = trans.stability_map;
        Ok(Transformation::new(
            trans.input_domain,
            trans.output_domain,
            Function::new_fallible(move |arg: &LazyFrame| {
                let res = function.eval(arg);
                res.map(|o| {
                    o.into_iter()
                        .map(AnyObject::new)
                        .collect::<Vec<AnyObject>>()
                })
            }),
            trans.input_metric,
            trans.output_metric,
            StabilityMap::new_fallible(move |d_in: &IntDistance| {
                let k = stability_map.eval(d_in)?;
                Ok(AnyObject::new(k))
            }),
        ))
        .into_any()
    }

    let null_partition = to_bool(null_partition);
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [
        (T, @????)
    ], (input_domain, partition_column, sum_column, bounds, null_partition))
} */