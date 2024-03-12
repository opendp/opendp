use std::path::PathBuf;

use polars::prelude::*;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{LazyFrameDomain, ParquetDomain},
    error::Fallible,
    transformations::DatasetMetric,
};

/// Sink a `LazyFrame` into a Parquet file.
///
/// # Arguments
/// * `input_domain` - `LazyFrameDomain`.
/// * `input_metric` - The metric under which neighboring `LazyFrame`s are compared.
/// * `path` - Path to the output Parquet file.
pub fn make_sink_parquet<M: DatasetMetric>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    path: PathBuf,
) -> Fallible<Transformation<LazyFrameDomain, ParquetDomain<LazyFrame>, M, M>>
where
    (ParquetDomain<LazyFrame>, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    let output_domain = ParquetDomain::new(input_domain.clone(), false, false, false);
    Transformation::new(
        input_domain.clone(),
        output_domain.clone(),
        Function::new_fallible(move |lazy_frame: &LazyFrame| {
            lazy_frame
                .clone()
                .sink_parquet(path.clone(), Default::default())?;
            Ok(path.clone())
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}
