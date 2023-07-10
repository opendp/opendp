use std::path::PathBuf;

use polars::prelude::*;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{DatasetMetric, LazyFrameDomain, ParquetDomain},
    error::Fallible,
};

pub fn make_sink_parquet<M: DatasetMetric>(
    input_domain: LazyFrameDomain,
    input_metric: M,
    path: PathBuf,
) -> Fallible<Transformation<LazyFrameDomain, ParquetDomain, M, M>>
where
    (ParquetDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    let output_domain = ParquetDomain::new(input_domain.clone());
    Transformation::new(
        input_domain.clone(),
        output_domain.clone(),
        Function::new_fallible(move |lazy_frame: &LazyFrame| {
            output_domain.write(path.clone(), lazy_frame.clone());
            Ok(Default::default())
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}