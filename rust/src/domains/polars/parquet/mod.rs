use std::path::PathBuf;

use polars::prelude::*;

use crate::domains::DatasetMetric;
use crate::{
    core::{Domain, MetricSpace},
    error::Fallible,
};

use super::LazyFrameDomain;

#[derive(Clone, PartialEq, Debug)]
pub struct ParquetDomain {
    pub lazyframe_domain: LazyFrameDomain,
}

impl ParquetDomain {
    pub fn new(lazyframe_domain: LazyFrameDomain) -> Self {
        ParquetDomain { lazyframe_domain }
    }

    pub fn read(
        &self,
        path: PathBuf,
        cache: bool,
        low_memory: bool,
        rechunk: bool,
    ) -> LazyFrame {
        let mut args = ScanArgsParquet::default();
        args.cache = cache;
        args.rechunk = rechunk;
        args.low_memory = low_memory;
        LazyFrame::scan_parquet(path, args).unwrap()
    }

    pub fn write(&self, path: PathBuf, lazy_frame: LazyFrame) {
        lazy_frame.sink_parquet(path, Default::default()).unwrap()
    }
}

impl Domain for ParquetDomain {
    type Carrier = PathBuf;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.lazyframe_domain
            .member(&self.read(val.clone(), false, false, false))
    }
}

impl<D: DatasetMetric> MetricSpace for (ParquetDomain, D) {
    fn check(&self) -> bool {
        true
    }
}
