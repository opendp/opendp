use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

use polars::prelude::*;

use crate::domains::{Frame, FrameDomain};
use crate::transformations::DatasetMetric;
use crate::{
    core::{Domain, MetricSpace},
    error::Fallible,
};

/// # Proof Definition
/// `ParquetDomain(F)` is the domain of all Parquet files holding data represented by `FrameDomain(F)``.
/// * `cache` - Cache the DataFrame after reading.
/// * `rechunk` - Rechunk the memory to contiguous chunks when parsing is done.
/// * `low_memory` - Reduce memory usage at the expense of performance
///
/// # Generics
/// * `F` - `LazyFrame` or `DataFrame`
///
/// # Example
/// ```
/// use opendp::domains::{AtomDomain, SeriesDomain, LazyFrameDomain, ParquetDomain};
///
/// let lazy_frame_domain = LazyFrameDomain::new(vec![
///             SeriesDomain::new("A", AtomDomain::<i32>::default()),
///             SeriesDomain::new("B", AtomDomain::<f64>::default()),
/// ])?;
///
/// let parquet_domain = ParquetDomain::new(lazy_frame_domain, false, false, false);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone)]
pub struct ParquetDomain<F: Frame> {
    pub frame_domain: FrameDomain<F>,
    pub cache: bool,
    pub rechunk: bool,
    pub low_memory: bool,
}

impl<F: Frame> ParquetDomain<F> {
    pub fn new(frame_domain: FrameDomain<F>, cache: bool, rechunk: bool, low_memory: bool) -> Self {
        ParquetDomain {
            frame_domain,
            cache,
            rechunk,
            low_memory,
        }
    }

    pub fn args(&self) -> ScanArgsParquet {
        let mut args = ScanArgsParquet::default();
        args.cache = self.cache;
        args.rechunk = self.rechunk;
        args.low_memory = self.low_memory;
        args
    }
}

impl<F: Frame> PartialEq for ParquetDomain<F> {
    fn eq(&self, other: &Self) -> bool {
        return self.frame_domain == other.frame_domain
            && self.cache == other.cache
            && self.rechunk == other.rechunk
            && self.low_memory == other.low_memory;
    }
}

impl<F: Frame> Debug for ParquetDomain<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ParquetDomain({})",
            self.frame_domain
                .series_domains
                .iter()
                .map(|s| format!("{}: {}", s.field.name, s.field.dtype))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

impl Domain for ParquetDomain<LazyFrame> {
    type Carrier = PathBuf;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.frame_domain
            .member(&LazyFrame::scan_parquet(val, self.args())?)
    }
}

impl Domain for ParquetDomain<DataFrame> {
    type Carrier = PathBuf;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.frame_domain
            .member(&LazyFrame::scan_parquet(val, self.args())?.collect()?)
    }
}

impl<D: DatasetMetric, F: Frame> MetricSpace for (ParquetDomain<F>, D)
where
    (FrameDomain<F>, D): MetricSpace,
{
    fn check_space(&self) -> Fallible<()> {
        (self.0.frame_domain.clone(), self.1.clone()).check_space()
    }
}
