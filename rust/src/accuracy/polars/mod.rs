use opendp_derive::bootstrap;
use polars::{
    datatypes::{AnyValue, DataType, Field},
    frame::{row::Row, DataFrame},
    prelude::{FunctionExpr, IntoLazy, LazyFrame, Schema},
};
use polars_plan::{
    dsl::{AggExpr, Expr},
    plans::DslPlan,
};

#[cfg(test)]
mod test;

use crate::{
    accuracy::{
        discrete_gaussian_scale_to_accuracy, discrete_laplacian_scale_to_accuracy,
        gaussian_scale_to_accuracy, laplacian_scale_to_accuracy,
    },
    core::{Measure, Measurement, Metric, MetricSpace},
    domains::LazyFrameDomain,
    error::Fallible,
    measurements::{
        expr_index_candidates::IndexCandidatesPlugin,
        expr_noise::{Distribution, NoisePlugin, Support},
        expr_report_noisy_max::ReportNoisyMaxPlugin,
        is_threshold_predicate, match_group_by, KeySanitizer, MatchGroupBy,
    },
    polars::{match_trusted_plugin, ExtractLazyFrame, OnceFrame},
    transformations::expr_discrete_quantile_score::DiscreteQuantileScorePlugin,
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    name = "summarize_polars_measurement",
    features("contrib"),
    arguments(
        measurement(rust_type = "AnyMeasurement"),
        alpha(c_type = "AnyObject *", default = b"null")
    ),
    generics(MI(suppress), MO(suppress)),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Summarize the statistics to be released from a measurement that returns a OnceFrame.
///
/// If a threshold is configured for censoring small/sensitive partitions,
/// a threshold column will be included,
/// containing the cutoff for the respective count query being thresholded.
///
/// # Arguments
/// * `measurement` - computation from which you want to read noise scale parameters from
/// * `alpha` - optional statistical significance to use to compute accuracy estimates
pub fn summarize_polars_measurement<MI: Metric, MO: 'static + Measure>(
    measurement: Measurement<LazyFrameDomain, OnceFrame, MI, MO>,
    alpha: Option<f64>,
) -> Fallible<DataFrame>
where
    (LazyFrameDomain, MI): MetricSpace,
{
    let schema = measurement.input_domain.schema();
    let lf = DataFrame::from_rows_and_schema(&[], &schema)?.lazy();
    let mut of = measurement.invoke(&lf)?;
    let lf: LazyFrame = of.eval_internal(&ExtractLazyFrame)?;

    summarize_lazyframe(&lf, alpha)
}

#[derive(Clone)]
struct UtilitySummary {
    pub name: String,
    pub aggregate: String,
    pub distribution: Option<String>,
    pub scale: Option<f64>,
    pub accuracy: Option<f64>,
    pub threshold: Option<u32>,
}

/// Summarize the statistics to be computed in a LazyFrame
///
/// # Arguments
/// * `lazyframe` - computation from which you want to read noise scale parameters from
/// * `alpha` - optional statistical significance to use to compute accuracy estimates
pub fn summarize_lazyframe(lazyframe: &LazyFrame, alpha: Option<f64>) -> Fallible<DataFrame> {
    let mut utility = summarize_logical_plan(&lazyframe.logical_plan, alpha)?;

    // only include the accuracy column if alpha is passed
    if alpha.is_none() {
        utility = utility.drop("accuracy")?;
    }
    // only include the threshold column if a threshold is set
    if utility.column("threshold")?.is_null().all() {
        utility = utility.drop("threshold")?;
    }
    Ok(utility)
}

/// Summarize the statistics to be computed in a LogicalPlan
fn summarize_logical_plan(logical_plan: &DslPlan, alpha: Option<f64>) -> Fallible<DataFrame> {
    if let Some(MatchGroupBy {
        aggs: exprs,
        key_sanitizer,
        ..
    }) = match_group_by(logical_plan.clone())?
    {
        let threshold = if let Some(KeySanitizer::Filter(predicate)) = key_sanitizer {
            Some(is_threshold_predicate(predicate.clone()).ok_or_else(|| {
                err!(
                    FailedFunction,
                    "predicate is not a valid filter: {}",
                    predicate
                )
            })?)
        } else {
            None
        };
        return agg_dataframe(&exprs, threshold, alpha);
    }
    if let DslPlan::Select { expr: exprs, .. } = logical_plan {
        return agg_dataframe(exprs, None, alpha);
    }

    if let DslPlan::Slice { input, .. }
    | DslPlan::Sink { input, .. }
    | DslPlan::HStack { input, .. } = logical_plan
    {
        return summarize_logical_plan(input.as_ref(), alpha);
    }

    fallible!(
        FailedFunction,
        "unrecognized dsl: {}",
        logical_plan.describe()?
    )
}

fn agg_dataframe(
    exprs: &Vec<Expr>,
    threshold: Option<(String, u32)>,
    alpha: Option<f64>,
) -> Fallible<DataFrame> {
    let rows = exprs
        .iter()
        .map(|e| {
            // ensures that the column name is right when summarizing columns with multiple statistics
            let name = e.clone().meta().output_name()?.to_string();
            Ok(summarize_expr(&e, alpha, threshold.clone())?
                .into_iter()
                .map(|mut summary| {
                    summary.name = name.clone();
                    summary
                })
                .collect())
        })
        .collect::<Fallible<Vec<Vec<UtilitySummary>>>>()?;

    Ok(DataFrame::from_rows_and_schema(
        &(rows.iter().flatten())
            .map(|summary| {
                Row(vec![
                    AnyValue::String(summary.name.as_ref()),
                    AnyValue::String(summary.aggregate.as_ref()),
                    match &summary.distribution {
                        Some(distribution) => AnyValue::String(distribution.as_ref()),
                        None => AnyValue::Null,
                    },
                    AnyValue::from(summary.scale),
                    AnyValue::from(summary.accuracy),
                    AnyValue::from(summary.threshold),
                ])
            })
            .collect::<Vec<_>>(),
        &Schema::from_iter(vec![
            Field::new("column".into(), DataType::String),
            Field::new("aggregate".into(), DataType::String),
            Field::new("distribution".into(), DataType::String),
            Field::new("scale".into(), DataType::Float64),
            Field::new("accuracy".into(), DataType::Float64),
            Field::new("threshold".into(), DataType::UInt32),
        ]),
    )?)
}
/// Summarize the statistics to be computed in an Expr
fn summarize_expr<'a>(
    expr: &Expr,
    alpha: Option<f64>,
    threshold: Option<(String, u32)>,
) -> Fallible<Vec<UtilitySummary>> {
    let name = expr.clone().meta().output_name()?.to_string();
    let expr = expr.clone().meta().undo_aliases();
    let t_value = threshold
        .clone()
        .and_then(|(t_name, t_value)| (name == t_name).then_some(t_value));

    if let Some((input, plugin)) = match_trusted_plugin::<NoisePlugin>(&expr)? {
        let accuracy = if let Some(alpha) = alpha {
            use {Distribution::*, Support::*};
            Some(match (plugin.distribution, plugin.support) {
                (Laplace, Float) => laplacian_scale_to_accuracy(plugin.scale, alpha),
                (Gaussian, Float) => gaussian_scale_to_accuracy(plugin.scale, alpha),
                (Laplace, Integer) => discrete_laplacian_scale_to_accuracy(plugin.scale, alpha),
                (Gaussian, Integer) => discrete_gaussian_scale_to_accuracy(plugin.scale, alpha),
            }?)
        } else {
            None
        };

        return Ok(vec![UtilitySummary {
            name,
            aggregate: expr_aggregate(&input[0])?.to_string(),
            distribution: Some(format!("{:?} {:?}", plugin.support, plugin.distribution)),
            scale: Some(plugin.scale),
            accuracy,
            threshold: t_value,
        }]);
    }

    // summarize quantile statistics
    if let Some((inputs, _)) = match_trusted_plugin::<IndexCandidatesPlugin>(&expr)? {
        return summarize_expr(&inputs[0], alpha, threshold);
    }

    if let Some((inputs, plugin)) = match_trusted_plugin::<ReportNoisyMaxPlugin>(&expr)? {
        return Ok(vec![UtilitySummary {
            name,
            aggregate: expr_aggregate(&inputs[0])?.to_string(),
            distribution: Some(format!("Gumbel{:?}", plugin.optimize)),
            scale: Some(plugin.scale),
            accuracy: None,
            threshold: t_value,
        }]);
    }

    Ok(match expr {
        Expr::Len => vec![UtilitySummary {
            name: name.clone(),
            aggregate: "Frame Length".to_string(),
            distribution: None,
            scale: None,
            accuracy: alpha.is_some().then_some(0.0),
            threshold: t_value,
        }],

        Expr::Function { input, .. } => input
            .iter()
            .map(|e| summarize_expr(e, alpha, threshold.clone()))
            .collect::<Fallible<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect(),

        Expr::BinaryExpr { left, op: _, right } => [
            summarize_expr(&left, alpha, threshold.clone())?,
            summarize_expr(&right, alpha, threshold)?,
        ]
        .concat(),

        e => return fallible!(FailedFunction, "unrecognized primitive: {:?}", e),
    })
}

fn expr_aggregate(expr: &Expr) -> Fallible<String> {
    if let Some((_, plugin)) = match_trusted_plugin::<DiscreteQuantileScorePlugin>(&expr)? {
        let (num, den) = plugin.alpha;
        return Ok(format!("{}-Quantile", num as f64 / den as f64));
    }
    Ok(match expr {
        Expr::Agg(AggExpr::Sum(_)) => "Sum",
        Expr::Len => "Frame Length",
        Expr::Agg(AggExpr::Count(_, include_null)) => {
            if *include_null {
                "Length"
            } else {
                "Count"
            }
        }
        Expr::Function {
            function: FunctionExpr::NullCount,
            ..
        } => "Null Count",
        Expr::Agg(AggExpr::NUnique(_)) => "N Unique",
        expr => return fallible!(FailedFunction, "unrecognized aggregation: {:?}", expr),
    }
    .to_string())
}
