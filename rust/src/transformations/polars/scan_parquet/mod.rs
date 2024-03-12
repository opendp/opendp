use polars::prelude::*;
use std::path::PathBuf;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{LazyFrameDomain, ParquetDomain},
    error::Fallible,
    transformations::DatasetMetric,
};

#[bootstrap(
    features("contrib"),
    arguments(
        cache(default = true),
        low_memory(default = false),
        rechunk(default = true),
    ),
    generics(M(suppress))
)]
/// Parse a path to a Parquet file into a `LazyFrame`.
///
/// # Arguments
/// * `input_domain` - Parquet domain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `cache` - Cache the `LazyFrame` after reading.
/// * `low_memory` - Reduce memory usage at the expense of performance
/// * `rechunk` - Rechunk the memory to contiguous chunks when parsing is done.
pub fn make_scan_parquet<M: DatasetMetric>(
    input_domain: ParquetDomain<LazyFrame>,
    input_metric: M,
) -> Fallible<Transformation<ParquetDomain<LazyFrame>, LazyFrameDomain, M, M>>
where
    (ParquetDomain<LazyFrame>, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        input_domain.frame_domain.clone(),
        Function::new_fallible(move |path: &PathBuf| {
            Ok(LazyFrame::scan_parquet(path, input_domain.args())?)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}
