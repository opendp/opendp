use std::path::PathBuf;

use polars::prelude::*;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{CsvDomain, DataFrameDomain, DatasetMetric},
    error::Fallible,
};

pub fn make_sink_csv<M: DatasetMetric>(
    input_domain: DataFrameDomain,
    input_metric: M,
    path: PathBuf,
) -> Fallible<Transformation<DataFrameDomain, CsvDomain<DataFrame>, M, M>>
where
    ( CsvDomain<DataFrame>, M): MetricSpace,
    (DataFrameDomain, M): MetricSpace,
{
    let output_domain =  CsvDomain::new(input_domain.clone());
    Transformation::new(
        input_domain.clone(),
        output_domain.clone(),
        Function::new_fallible(move |data_frame: &DataFrame| {
            output_domain
                .new_writer(path.clone())
                .finish(&mut data_frame.clone())?;
            Ok(Default::default())
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}