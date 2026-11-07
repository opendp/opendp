//! Exact scoring of a sparse SQL histogram, executed only in the continuation.
use std::sync::Arc;

use polars::{
    datatypes::{ArrayChunked, ArrowDataType, DataType, Field},
    error::{PolarsResult, polars_bail, polars_err},
    prelude::{AnonymousColumnsUdf, Column, IntoColumn},
    series::Series,
};
use polars_arrow::{
    array::{FixedSizeListArray, UInt64Array},
    datatypes::Field as ArrowField,
};
use polars_plan::{dsl::ColumnsUdf, prelude::FunctionOptions};
#[cfg(feature = "ffi")]
use serde::{Deserialize, Serialize};

use crate::{polars::OpenDPPlugin, transformations::score_candidate_counts};

#[derive(Clone)]
#[cfg_attr(feature = "ffi", derive(Serialize, Deserialize))]
pub(crate) struct HistogramQuantileScorePlugin {
    pub num_candidates: usize,
    pub alpha: (u64, u64),
    pub size_limit: u64,
}

impl OpenDPPlugin for HistogramQuantileScorePlugin {
    const NAME: &'static str = "histogram_quantile_score_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }
}

impl AnonymousColumnsUdf for HistogramQuantileScorePlugin {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }
    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }
    fn get_field(&self, _: &polars::prelude::Schema, fields: &[Field]) -> PolarsResult<Field> {
        histogram_quantile_score_type(fields, self.clone())
    }
}

impl ColumnsUdf for HistogramQuantileScorePlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn call_udf(&self, inputs: &mut [Column]) -> PolarsResult<Column> {
        let [buckets, counts] = inputs else {
            polars_bail!(InvalidOperation: "histogram scorer expects bucket and count columns")
        };
        if buckets.len() != counts.len() || buckets.null_count() != 0 || counts.null_count() != 0 {
            polars_bail!(InvalidOperation: "histogram columns must have equal lengths and no nulls")
        }
        let mut histogram = vec![0u64; 2 * self.num_candidates + 1];
        for (bucket, count) in buckets
            .u64()?
            .into_no_null_iter()
            .zip(counts.u64()?.into_no_null_iter())
        {
            let cell = histogram
                .get_mut(bucket as usize)
                .ok_or_else(|| polars_err!(InvalidOperation: "histogram bucket out of range"))?;
            *cell += count;
        }
        let scores = UInt64Array::from_values(
            histogram_scores(&histogram, self.alpha, self.size_limit).into_iter(),
        );
        let dtype = ArrowDataType::FixedSizeList(
            Box::new(ArrowField::new("item".into(), ArrowDataType::UInt64, true)),
            self.num_candidates,
        );
        Ok(Series::from(ArrayChunked::from(FixedSizeListArray::new(
            dtype,
            1,
            Box::new(scores),
            None,
        )))
        .into_column())
    }
}

pub(crate) fn histogram_scores(histogram: &[u64], alpha: (u64, u64), size_limit: u64) -> Vec<u64> {
    let n = histogram.iter().sum();
    let mut prefix = 0;
    let counts = histogram.chunks_exact(2).map(move |pair| {
        let lt = prefix + pair[0];
        prefix = lt + pair[1];
        (lt, prefix)
    });
    score_candidate_counts(n, counts, alpha.0, alpha.1, size_limit).collect()
}

pub(crate) fn histogram_quantile_score_type(
    fields: &[Field],
    kwargs: HistogramQuantileScorePlugin,
) -> PolarsResult<Field> {
    let [bucket, count] = fields else {
        polars_bail!(InvalidOperation: "histogram scorer expects two columns")
    };
    if bucket.dtype() != &DataType::UInt64 || count.dtype() != &DataType::UInt64 {
        polars_bail!(InvalidOperation: "histogram scorer expects UInt64 buckets and counts")
    }
    Ok(Field::new(
        bucket.name().clone(),
        DataType::Array(Box::new(DataType::UInt64), kwargs.num_candidates),
    ))
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type_func_with_kwargs=histogram_quantile_score_type)]
fn histogram_quantile_score_plugin(
    inputs: &[Series],
    kwargs: HistogramQuantileScorePlugin,
) -> PolarsResult<Series> {
    let mut inputs: Vec<Column> = inputs
        .iter()
        .cloned()
        .map(IntoColumn::into_column)
        .collect();
    Ok(kwargs.call_udf(&mut inputs)?.take_materialized_series())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transformations::score_candidates_constants;

    #[test]
    fn score_tails_are_clipped_only_after_exact_prefixes() -> crate::error::Fallible<()> {
        let histograms = [
            vec![0u64, 0, 0, 0, 0],
            vec![4, 7, 2, 3, 5],
            vec![u64::MAX - 4, 1, 1, 1, 1],
            vec![0, u64::MAX, 0],
        ];
        for histogram in histograms {
            let n = histogram.iter().map(|&v| v as u128).sum::<u128>();
            let calibrated = score_candidates_constants(Some(u64::MAX / 4), 0.34434532242)?;
            for (num, den, limit) in [
                (0, 1, u64::MAX),
                (1, 1, u64::MAX),
                (1, 2, u64::MAX / 2),
                (1, 2, 0),
                (1, 2, 3),
                calibrated,
            ] {
                let mut prefix = 0u128;
                let expected = histogram
                    .chunks_exact(2)
                    .map(|pair| {
                        let lt = prefix + pair[0] as u128;
                        prefix = lt + pair[1] as u128;
                        let gt = n - prefix;
                        let a = (den - num) as u128 * lt.min(limit as u128);
                        let b = num as u128 * gt.min(limit as u128);
                        u64::try_from(a.abs_diff(b)).unwrap()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(histogram_scores(&histogram, (num, den), limit), expected);
            }
        }
        Ok(())
    }
}
