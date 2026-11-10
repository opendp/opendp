use opendp_derive::bootstrap;
use polars::prelude::{DataFrame, LazyFrame};

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{DataFrameDomain, LazyFrameDomain},
    error::Fallible,
};

#[cfg(feature = "ffi")]
mod ffi;

/// Materialize an in-domain Polars lazy plan as an eager DataFrame.
///
/// # Arguments
/// * `input_domain` - Domain of the input LazyFrame.
/// * `input_metric` - Dataset adjacency relation.
#[bootstrap(features("contrib"), generics(M(suppress)))]
pub fn make_collect<M: Metric>(
    input_domain: LazyFrameDomain,
    input_metric: M,
) -> Fallible<Transformation<LazyFrameDomain, M, DataFrameDomain, M>>
where
    (LazyFrameDomain, M): MetricSpace,
    (DataFrameDomain, M): MetricSpace,
    M::Distance: Clone + 'static,
{
    Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        input_domain.cast_carrier(),
        input_metric,
        Function::new_fallible(|frame: &LazyFrame| {
            frame.clone().collect().map_err(Into::into)
        }),
        StabilityMap::new(Clone::clone),
    )
}

/// Convert an eager Polars DataFrame to a lazy plan without executing data.
///
/// # Arguments
/// * `input_domain` - Domain of the input DataFrame.
/// * `input_metric` - Dataset adjacency relation.
#[bootstrap(features("contrib"), generics(M(suppress)))]
pub fn make_lazy<M: Metric>(
    input_domain: DataFrameDomain,
    input_metric: M,
) -> Fallible<Transformation<DataFrameDomain, M, LazyFrameDomain, M>>
where
    (DataFrameDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
    M::Distance: Clone + 'static,
{
    Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        input_domain.cast_carrier(),
        input_metric,
        Function::new(|frame: &DataFrame| frame.clone().lazy()),
        StabilityMap::new(Clone::clone),
    )
}
