use polars::prelude::LazyFrame;

use crate::{
    core::{FfiResult, MetricSpace, IntoAnyTransformationFfiResultExt},
    domains::{CsvDomain, DatasetMetric, LazyFrameDomain},
    ffi::{
        any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
        util::{self, c_bool},
    },
    metrics::{InsertDeleteDistance, SymmetricDistance},
};


#[no_mangle]
pub extern "C" fn opendp_transformations__make_scan_csv(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    cache: c_bool,
    low_memory: c_bool,
    rechunk: c_bool,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain =
        try_!(try_as_ref!(input_domain).downcast_ref::<CsvDomain<LazyFrame>>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let cache: bool = util::to_bool(cache);
    let low_memory: bool = util::to_bool(low_memory);
    let rechunk: bool = util::to_bool(rechunk);
    
    let M = input_metric.type_.clone();

    fn monomorphize<M: 'static + DatasetMetric>(
        input_domain: CsvDomain<LazyFrame>,
        input_metric: &AnyMetric,
        cache: bool,
        low_memory: bool,
        rechunk: bool,
    ) -> FfiResult<*mut AnyTransformation>
    where
        (CsvDomain<LazyFrame>, M): MetricSpace,
        (LazyFrameDomain, M): MetricSpace,
    {
        let input_metric: M = try_!(input_metric.downcast_ref::<M>()).clone();
        super::make_scan_csv(input_domain, input_metric, cache, low_memory, rechunk).into_any()
    }

    dispatch!(
        monomorphize,
        [(M, [SymmetricDistance, InsertDeleteDistance])],
        (
            input_domain,
            input_metric,
            cache,
            low_memory,
            rechunk
        )
    )
}
