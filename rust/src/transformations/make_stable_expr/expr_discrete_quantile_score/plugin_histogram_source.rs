//! Logical sparse histogram aggregate. SQL binning belongs to the translator.
use std::sync::Arc;

use polars::{
    datatypes::{
        Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, StaticArray,
        UInt32Type, UInt64Type,
    },
    error::{PolarsResult, polars_bail, polars_err},
    prelude::{
        AnonymousColumnsUdf, Column, DataType, DslPlan, Expr, Field, IntoColumn, IntoSeries,
        NamedFrom, PlSmallStr, PolarsPhysicalType, StructChunked, cols,
    },
    series::Series,
};
use polars_plan::{
    dsl::ColumnsUdf,
    prelude::{DslBuilder, DslFunction, FunctionOptions},
};
#[cfg(feature = "ffi")]
use serde::{Deserialize, Serialize};

use super::{DiscreteQuantileScorePlugin, series_to_vec};
use crate::polars::OpenDPPlugin;

#[derive(Clone)]
#[cfg_attr(feature = "ffi", derive(Serialize, Deserialize))]
pub(crate) struct DiscreteQuantileHistogramPlugin {
    pub scorer: DiscreteQuantileScorePlugin,
    pub bucket_name: PlSmallStr,
    pub count_name: PlSmallStr,
}

impl OpenDPPlugin for DiscreteQuantileHistogramPlugin {
    const NAME: &'static str = "discrete_quantile_histogram_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::groupwise()
    }
}
impl AnonymousColumnsUdf for DiscreteQuantileHistogramPlugin {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }
    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }
    fn get_field(&self, _: &polars::prelude::Schema, fields: &[Field]) -> PolarsResult<Field> {
        discrete_quantile_histogram_type(fields, self.clone())
    }
}
impl ColumnsUdf for DiscreteQuantileHistogramPlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn call_udf(&self, inputs: &mut [Column]) -> PolarsResult<Column> {
        let [input] = inputs else {
            polars_bail!(InvalidOperation: "histogram expects one input")
        };
        let series = input.as_materialized_series();
        let counts = match series.dtype() {
            DataType::UInt32 => histogram::<UInt32Type>(series, &self.scorer),
            DataType::UInt64 => histogram::<UInt64Type>(series, &self.scorer),
            DataType::Int8 => histogram::<Int8Type>(series, &self.scorer),
            DataType::Int16 => histogram::<Int16Type>(series, &self.scorer),
            DataType::Int32 => histogram::<Int32Type>(series, &self.scorer),
            DataType::Int64 => histogram::<Int64Type>(series, &self.scorer),
            DataType::Float32 => histogram::<Float32Type>(series, &self.scorer),
            DataType::Float64 => histogram::<Float64Type>(series, &self.scorer),
            dtype => {
                polars_bail!(InvalidOperation: "Expected supported numeric histogram input, found {dtype}")
            }
        }?;
        let (mut buckets, mut counts): (Vec<u64>, Vec<u64>) = counts
            .into_iter()
            .enumerate()
            .filter_map(|(bucket, count)| (count != 0).then_some((bucket as u64, count)))
            .unzip();
        // Keep observed groups even if their expression contributed no values.
        if buckets.is_empty() {
            buckets.push(0);
            counts.push(0);
        }
        let fields = [
            Series::new(self.bucket_name.clone(), buckets),
            Series::new(self.count_name.clone(), counts),
        ];
        Ok(
            StructChunked::from_series(input.name().clone(), fields[0].len(), fields.iter())?
                .into_series()
                .into_column(),
        )
    }
}

fn histogram<PT: 'static + PolarsPhysicalType>(
    series: &Series,
    scorer: &DiscreteQuantileScorePlugin,
) -> PolarsResult<Vec<u64>>
where
    for<'a> PT::Physical<'a>: 'static + PartialOrd,
    PT::Array: StaticArray,
{
    let candidates = scorer.candidates.cast(&PT::get_static_dtype())?;
    let candidates =
        series_to_vec::<PT>(&candidates).map_err(|e| polars_err!(InvalidOperation: "{e}"))?;
    let mut counts = vec![0u64; 2 * candidates.len() + 1];
    for value in series
        .unpack::<PT>()?
        .downcast_iter()
        .flat_map(StaticArray::values_iter)
    {
        let lt = candidates.partition_point(|c| *c < value);
        let le = lt + candidates[lt..].partition_point(|c| *c == value);
        counts[lt + le] += 1;
    }
    Ok(counts)
}

pub(crate) fn discrete_quantile_histogram_type(
    fields: &[Field],
    kwargs: DiscreteQuantileHistogramPlugin,
) -> PolarsResult<Field> {
    let [input] = fields else {
        polars_bail!(InvalidOperation: "histogram expects one input")
    };
    Ok(Field::new(
        input.name().clone(),
        DataType::Struct(vec![
            Field::new(kwargs.bucket_name, DataType::UInt64),
            Field::new(kwargs.count_name, DataType::UInt64),
        ]),
    ))
}

/// Flatten only the sparse aggregate result, never a list of private observations.
/// The SQL translator lowers this logical relation directly to COUNT GROUP BY.
pub(crate) fn sparse_histogram_plan(mut plan: DslPlan, source: Expr, name: PlSmallStr) -> DslPlan {
    let grouped = matches!(plan, DslPlan::GroupBy { .. });
    match &mut plan {
        DslPlan::Select { expr, .. } | DslPlan::GroupBy { aggs: expr, .. } => {
            *expr = vec![source.alias(name.clone())]
        }
        _ => unreachable!("called at a private Select/GroupBy"),
    }
    let mut builder = DslBuilder::from(plan);
    if grouped {
        builder = builder.explode(
            cols([name.clone()]),
            polars::prelude::ExplodeOptions {
                empty_as_null: true,
                keep_nulls: true,
            },
            false,
        );
    }
    builder
        .map_private(DslFunction::Unnest {
            columns: cols([name]),
            separator: None,
        })
        .0
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type_func_with_kwargs=discrete_quantile_histogram_type)]
fn discrete_quantile_histogram_plugin(
    inputs: &[Series],
    kwargs: DiscreteQuantileHistogramPlugin,
) -> PolarsResult<Series> {
    let mut inputs = inputs
        .iter()
        .cloned()
        .map(IntoColumn::into_column)
        .collect::<Vec<_>>();
    Ok(kwargs.call_udf(&mut inputs)?.take_materialized_series())
}
