use std::path::PathBuf;

use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{CsvDomain, LazyFrameDomain},
    error::Fallible,
    transformations::DatasetMetric,
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
        cache(default = true),
        low_memory(default = false),
        rechunk(default = true),
    ),
    generics(M(suppress))
)]
/// Parse a path to a CSV into a LazyFrame.
///
/// # Arguments
/// * `input_domain` - CsvDomain(LazyFrame)
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `cache` - Cache the DataFrame after reading.
/// * `low_memory` - Reduce memory usage at the expense of performance
/// * `rechunk` - Rechunk the memory to contiguous chunks when parsing is done.
pub fn make_scan_csv<M: DatasetMetric>(
    input_domain: CsvDomain<LazyFrame>,
    input_metric: M,
    cache: bool,
    low_memory: bool,
    rechunk: bool,
) -> Fallible<Transformation<CsvDomain<LazyFrame>, LazyFrameDomain, M, M>>
where
    (CsvDomain<LazyFrame>, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        input_domain.frame_domain.clone(),
        Function::new_fallible(move |path: &PathBuf| {
            Ok(input_domain
                .new_reader(path.clone())
                .with_cache(cache)
                .low_memory(low_memory)
                .with_rechunk(rechunk)
                .finish()?)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}
