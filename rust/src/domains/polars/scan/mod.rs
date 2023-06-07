use std::path::PathBuf;

use polars::prelude::*;

use crate::{
    core::{Domain, Metric, MetricSpace},
    error::Fallible,
    metrics::{
        ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
    },
};

use super::LazyFrameDomain;

pub trait DatasetMetric: Metric<Distance = IntDistance> {}
impl DatasetMetric for SymmetricDistance {}
impl DatasetMetric for InsertDeleteDistance {}
impl DatasetMetric for ChangeOneDistance {}
impl DatasetMetric for HammingDistance {}

pub struct CsvDomain {
    pub lazy_frame_domain: LazyFrameDomain,
    pub reader: LazyCsvReader<'static>,
}

impl CsvDomain {
    pub fn new(lazy_frame_domain: LazyFrameDomain, reader: LazyCsvReader<'static>) -> Self {
        CsvDomain {
            lazy_frame_domain,
            reader,
        }
    }
}

impl Domain for CsvDomain {
    type Carrier = String;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.lazy_frame_domain.member(
            &(self.reader.clone())
                .with_path(PathBuf::from(val).clone())
                .finish()?,
        )
    }
}

impl PartialEq for CsvDomain {
    fn eq(&self, other: &Self) -> bool {
        self.lazy_frame_domain == other.lazy_frame_domain
    }
}

impl std::fmt::Debug for CsvDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CsvDomain({:?})", self.lazy_frame_domain)
    }
}

impl Clone for CsvDomain {
    fn clone(&self) -> Self {
        Self {
            lazy_frame_domain: self.lazy_frame_domain.clone(),
            reader: self.reader.clone(),
        }
    }
}

impl<D: DatasetMetric> MetricSpace for (CsvDomain, D) {
    fn check(&self) -> bool {
        true
    }
}

pub struct ParquetDomain {
    pub lazy_frame_domain: LazyFrameDomain,
    pub scan_args_parquet: ScanArgsParquet,
}

impl Domain for ParquetDomain {
    type Carrier = String;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.lazy_frame_domain.member(&LazyFrame::scan_parquet(
            PathBuf::from(val),
            self.scan_args_parquet.clone(),
        )?)
    }
}

impl PartialEq for ParquetDomain {
    fn eq(&self, other: &Self) -> bool {
        self.lazy_frame_domain == other.lazy_frame_domain
    }
}

impl std::fmt::Debug for ParquetDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParquetDomain({:?})", self.lazy_frame_domain)
    }
}

impl Clone for ParquetDomain {
    fn clone(&self) -> Self {
        Self {
            lazy_frame_domain: self.lazy_frame_domain.clone(),
            scan_args_parquet: self.scan_args_parquet.clone(),
        }
    }
}

impl<D: DatasetMetric> MetricSpace for (ParquetDomain, D) {
    fn check(&self) -> bool {
        true
    }
}
