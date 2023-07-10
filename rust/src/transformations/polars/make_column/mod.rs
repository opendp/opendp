use opendp_derive::bootstrap;
use polars::prelude::DataFrame;

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{DataFrameDomain, SeriesDomain},
    error::Fallible,
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib"), generics(M(suppress)))]
/// Extract a Series from a DataFrame
pub fn make_column<M: Metric>(
    input_domain: DataFrameDomain,
    input_metric: M,
    column_name: String,
) -> Fallible<Transformation<DataFrameDomain, SeriesDomain, M, M>>
where
    M::Distance: 'static + Clone,
    (DataFrameDomain, M): MetricSpace,
    (SeriesDomain, M): MetricSpace,
{
    let output_domain = input_domain.try_column(&column_name)?.clone();
    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(move |arg: &DataFrame| {
            arg.clone()
                .drop_in_place(column_name.as_str())
                .map_err(Into::into)
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(M::Distance::clone),
    )
}
