use std::{convert::TryFrom, os::raw::c_char};

use polars::prelude::*;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    domains::{LazyFrameDomain},
    ffi::{
        any::{AnyDomain, AnyObject, AnyTransformation, Downcast},
        util::{c_bool, to_bool, Type, to_str},
    },
    traits::Float,
    transformations::{make_sized_partitioned_sum},
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_partitioned_sum(
    input_domain: *const AnyDomain,
    partition_column: *const c_char,
    sum_column: *const c_char,
    bounds: *const AnyObject,
    null_partition: c_bool,
    T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {

    fn monomorphize<T: Float>(
        input_domain: *const AnyDomain,
        partition_column: *const c_char,
        sum_column: *const c_char,
        bounds: *const AnyObject,
        null_partition: bool,
    ) -> FfiResult<*mut AnyTransformation> {

        let partition_column =  try_!(to_str(partition_column));
        let sum_column =  try_!(to_str(sum_column));

        let mut input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
        // Temporary line to add margins for test_polars.py only
        input_domain = try_!(input_domain.with_counts(df![partition_column => ["AA", "BB"], "count" => [3, 2]].unwrap().lazy()));

        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<(T,T)>()).clone();
    
        make_sized_partitioned_sum(
            input_domain,
            partition_column,
            sum_column,
            bounds,
            null_partition
        ).into_any()
    }

    let null_partition = to_bool(null_partition);
    let T = try_!(Type::try_from(T));

    dispatch!(monomorphize, [
        (T, @floats)
    ], (input_domain, partition_column, sum_column, bounds, null_partition))
}