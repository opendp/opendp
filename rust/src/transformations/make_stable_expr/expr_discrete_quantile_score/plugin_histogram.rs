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
    use crate::error::Fallible;
    use crate::{
        measurements::expr_noisy_max::NoisyMaxPlugin,
        polars::{apply_anonymous_function, split::split_plan},
        transformations::{
            expr_discrete_quantile_score::DiscreteQuantileScorePlugin, score_candidates_constants,
        },
    };
    use polars::prelude::*;

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

    #[test]
    fn typed_histograms_match_original_scores() -> Fallible<()> {
        let mut cases: Vec<(Series, Series)> = vec![];
        for candidates in [vec![0i64], vec![-3, 0, 2], vec![-4, -1, 0, 1, 5, 9]] {
            cases.push((
                Series::new(
                    "value".into(),
                    [-8i64, -4, -4, -3, -2, -1, 0, 0, 1, 2, 3, 5, 9, 12],
                ),
                Series::new("".into(), candidates),
            ));
        }
        cases.extend([
            (
                Series::new(
                    "value".into(),
                    [0u64, 1 << 53, (1 << 53) + 1, u64::MAX - 1, u64::MAX],
                ),
                Series::new("".into(), [1 << 53, (1 << 53) + 1, u64::MAX - 1]),
            ),
            (
                Series::new("value".into(), [i64::MIN, i64::MIN + 1, -1, 0, i64::MAX]),
                Series::new("".into(), [i64::MIN + 1, 0, i64::MAX]),
            ),
            (
                Series::new(
                    "value".into(),
                    [f64::NEG_INFINITY, -1., -0., 0., 1., f64::INFINITY],
                ),
                Series::new("".into(), [-1f64, 0., 1.]),
            ),
            (
                Series::new(
                    "value".into(),
                    [f32::NEG_INFINITY, -1., -0., 0., 1., f32::INFINITY],
                ),
                Series::new("".into(), [-1f32, 0., 1.]),
            ),
            (
                Series::new("value".into(), Vec::<i64>::new()),
                Series::new("".into(), [-1i64, 0, 1]),
            ),
            (
                Series::new(
                    "value".into(),
                    [Some(-1f64), None, Some(f64::NAN), Some(1.)],
                ),
                Series::new("".into(), [-1f64, 0., 1.]),
            ),
        ]);
        for dtype in [DataType::Int8, DataType::Int16, DataType::Int32] {
            cases.push((
                Series::new("value".into(), [-8i64, -3, -1, 0, 0, 1, 2, 12]).cast(&dtype)?,
                Series::new("".into(), [-3i64, 0, 2]).cast(&dtype)?,
            ));
        }
        cases.push((
            Series::new("value".into(), [0u32, 1, 2, 2, 3, u32::MAX]),
            Series::new("".into(), [0u32, 2, u32::MAX]),
        ));
        for (values, candidates) in cases {
            let input = DataFrame::new(vec![values.clone().into_column()])?.lazy();
            let mut expr = col("value").fill_null(lit(0));
            if values.dtype().is_float() {
                expr = expr.fill_nan(lit(0));
            }
            let scorer = DiscreteQuantileScorePlugin {
                candidates: candidates.clone(),
                alpha: (2, 7),
                size_limit: 1,
            };
            let scores = apply_anonymous_function(vec![expr.clone()], scorer);
            let query = input.clone().select([apply_anonymous_function(
                vec![scores],
                NoisyMaxPlugin {
                    replacement: false,
                    scale: 0.,
                    negate: true,
                },
            )]);
            let plans = split_plan(query.logical_plan, std::path::Path::new("unused"))?;
            let DslPlan::GroupBy { keys, aggs, .. } = &plans[0] else {
                panic!()
            };
            let bucket = keys.last().unwrap().clone().meta().output_name()?;
            let count = aggs[0].clone().meta().output_name()?;
            let histogram = LazyFrame::from(plans[0].clone()).collect()?;
            let projected = DataFrame::new(vec![values.clone().into_column()])?
                .lazy()
                .select([expr.clone().cast(candidates.dtype().clone()).alias("value")])
                .collect()?;
            let mut expected_counts = vec![0u64; 2 * candidates.len() + 1];
            for value in projected.column("value")?.as_materialized_series().iter() {
                let less = candidates
                    .iter()
                    .filter(|c| c.partial_cmp(&value) == Some(std::cmp::Ordering::Less))
                    .count();
                let equal = candidates.iter().filter(|c| *c == value).count();
                expected_counts[2 * less + equal] += 1;
            }
            let mut actual_counts = vec![0u64; expected_counts.len()];
            for (bucket, count) in histogram
                .column(&bucket)?
                .u64()?
                .into_no_null_iter()
                .zip(histogram.column(&count)?.u64()?.into_no_null_iter())
            {
                assert!(count > 0, "no scaffold rows");
                actual_counts[bucket as usize] = count;
            }
            assert_eq!(
                actual_counts,
                expected_counts,
                "exact typed buckets: {:?}",
                values.dtype()
            );
            if values.is_empty() {
                assert_eq!(histogram.height(), 0);
            }
            for alpha in [(0, 1), (1, 2), (1, 1), (2, 7)] {
                for size_limit in [0u64, 1, 100] {
                    let expected = input
                        .clone()
                        .select([apply_anonymous_function(
                            vec![expr.clone()],
                            DiscreteQuantileScorePlugin {
                                candidates: candidates.clone(),
                                alpha,
                                size_limit,
                            },
                        )
                        .alias("score")])
                        .collect()?;
                    let score = apply_anonymous_function(
                        vec![col(bucket.clone()), col(count.clone())],
                        HistogramQuantileScorePlugin {
                            num_candidates: candidates.len(),
                            alpha,
                            size_limit,
                        },
                    )
                    .alias("score");
                    let actual = histogram.clone().lazy().select([score]).collect()?;
                    assert_eq!(
                        actual,
                        expected,
                        "dtype {:?}, alpha {alpha:?}, limit {size_limit}",
                        values.dtype()
                    );
                }
            }
        }
        Ok(())
    }
}
