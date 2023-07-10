use std::path::PathBuf;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{DatasetMetric, LazyFrameDomain, ParquetDomain},
    error::Fallible,
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
/// Parse a path to a Parquet file into a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Parquet domain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `cache` - Cache the DataFrame after reading.
/// * `low_memory` - Reduce memory usage at the expense of performance
/// * `rechunk` - Rechunk the memory to contiguous chunks when parsing is done.
pub fn make_scan_parquet<M: DatasetMetric>(
    input_domain: ParquetDomain,
    input_metric: M,
    cache: bool,
    low_memory: bool,
    rechunk: bool,
) -> Fallible<Transformation<ParquetDomain, LazyFrameDomain, M, M>>
where
    (ParquetDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        input_domain.lazyframe_domain.clone(),
        Function::new_fallible(move |path: &PathBuf| {
            Ok(input_domain.read(path.clone(), cache, low_memory, rechunk))
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}
