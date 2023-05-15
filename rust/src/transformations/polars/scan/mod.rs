use std::{fs::File, path::PathBuf};

use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, MetricSpace, StabilityMap, Transformation},
    domains::{CsvDomain, DatasetMetric, LazyFrameDomain, ParquetDomain},
    error::Fallible,
};

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_scan_csv<M: DatasetMetric>(
    input_domain: CsvDomain,
    input_metric: M,
) -> Fallible<Transformation<CsvDomain, LazyFrameDomain, M, M>>
where
    (CsvDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    let CsvDomain {
        lazy_frame_domain,
        reader,
    } = input_domain.clone();

    Transformation::new(
        input_domain,
        lazy_frame_domain,
        Function::new_fallible(move |path: &String| {
            let reader = reader.clone().with_path(PathBuf::from(path).clone());

            Ok(reader.finish()?)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}

/// Write a `LazyFrame` to a CSV file.
/// 
/// # Arguments
/// 
#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *", rust_type = b"null"),
        input_metric(c_type = "AnyMetric *"),
        output_path(c_type = "char *", rust_type = b"null"),
        MI(example = "input_metric"),
    )
)]
pub fn make_sink_csv<MI: DatasetMetric>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    output_path: String,
) -> Fallible<Transformation<LazyFrameDomain, CsvDomain, MI, MI>>
where
    (LazyFrameDomain, MI): MetricSpace,
    (CsvDomain, MI): MetricSpace,
{
    Transformation::<LazyFrameDomain, CsvDomain, MI, MI>::new(
        input_domain.clone(),
        CsvDomain { lazy_frame_domain: input_domain, reader: LazyCsvReader::new("") },
        Function::new_fallible(move |frame: &LazyFrame| {
            println!("output_path: {}", output_path);
            let output_file: File = File::create(PathBuf::from(output_path.clone())).unwrap();

            CsvWriter::new(output_file).finish(&mut frame.clone().collect()?)?;
            Ok(output_path.clone())
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}

pub fn make_scan_parquet<M: DatasetMetric>(
    input_domain: ParquetDomain,
    input_metric: M,
) -> Fallible<Transformation<ParquetDomain, LazyFrameDomain, M, M>>
where
    (ParquetDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    let ParquetDomain {
        lazy_frame_domain,
        scan_args_parquet,
    } = input_domain.clone();

    Transformation::new(
        input_domain,
        lazy_frame_domain,
        Function::new_fallible(move |path: &String| {
            Ok(LazyFrame::scan_parquet(PathBuf::from(path), scan_args_parquet.clone())?)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}
