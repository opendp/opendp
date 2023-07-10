use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{DataFrameDomain, LazyFrameDomain, Margin},
    error::Fallible,
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib"), generics(M(suppress)))]
/// Converts a DataFrame to a LazyFrame
pub fn make_lazy<M: Metric>(
    input_domain: DataFrameDomain,
    input_metric: M,
) -> Fallible<Transformation<DataFrameDomain, LazyFrameDomain, M, M>>
where
    M::Distance: 'static + Clone,
    (DataFrameDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    Transformation::new(
        input_domain.clone(),
        LazyFrameDomain {
            series_domains: input_domain.series_domains,
            margins: (input_domain.margins.into_iter())
                .map(|(k, m)| {
                    let margin = Margin {
                        data: m.data.lazy(),
                        counts: m.counts,
                    };
                    Ok((k, margin))
                })
                .collect::<Fallible<_>>()?,
        },
        Function::new(|arg: &DataFrame| arg.clone().lazy()),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(M::Distance::clone),
    )
}
