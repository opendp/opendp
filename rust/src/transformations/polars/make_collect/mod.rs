use opendp_derive::bootstrap;
use polars::prelude::LazyFrame;

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{DataFrameDomain, LazyFrameDomain, Margin},
    error::Fallible,
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib"), generics(M(suppress)))]
/// Converts a LazyFrame to a DataFrame
pub fn make_collect<M: Metric>(
    input_domain: LazyFrameDomain,
    input_metric: M,
) -> Fallible<Transformation<LazyFrameDomain, DataFrameDomain, M, M>>
where
    M::Distance: 'static + Clone,
    (LazyFrameDomain, M): MetricSpace,
    (DataFrameDomain, M): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        DataFrameDomain {
            series_domains: input_domain.series_domains,
            margins: (input_domain.margins.into_iter())
                .map(|(k, m)| {
                    let margin = Margin {
                        data: m.data.collect()?,
                        counts_index: m.counts_index,
                    };
                    Ok((k, margin))
                })
                .collect::<Fallible<_>>()?,
        },
        Function::new_fallible(|arg: &LazyFrame| {
            arg.clone().collect().map_err(Into::into)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(M::Distance::clone),
    )
}
