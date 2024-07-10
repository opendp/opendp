use opendp_derive::bootstrap;
use polars::{
    datatypes::{AnyValue, DataType, Field},
    frame::{row::Row, DataFrame},
    prelude::{IntoLazy, LazyFrame, Schema},
};
use polars_plan::{
    dsl::{AggExpr, Expr},
    plans::DslPlan,
};

use crate::{
    accuracy::{
        discrete_gaussian_scale_to_accuracy, discrete_laplacian_scale_to_accuracy,
        gaussian_scale_to_accuracy, laplacian_scale_to_accuracy,
    },
    core::{Measure, Measurement, Metric, MetricSpace},
    domains::LazyFrameDomain,
    error::Fallible,
    measurements::{
        expr_noise::{match_noise, Distribution, Support},
        expr_report_noisy_max_gumbel::match_report_noisy_max_gumbel,
    },
    polars::{ExtractLazyFrame, OnceFrame},
    transformations::expr_discrete_quantile_score::match_discrete_quantile_score,
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    name = "describe_onceframe_measurement_accuracy",
    features("contrib"),
    arguments(
        measurement(rust_type = "AnyMeasurement"),
        alpha(c_type = "AnyObject *", default = b"null")
    ),
    generics(MI(suppress), MO(suppress)),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Get noise scale parameters from a measurement that returns a OnceFrame.
///
/// # Arguments
/// * `measurement` - computation from which you want to read noise scale parameters from
/// * `alpha` - optional statistical significance to use to compute accuracy estimates
pub fn describe_onceframe_measurement_accuracy<MI: Metric, MO: 'static + Measure>(
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

    lazyframe_utility(&lf, alpha)
}

struct UtilitySummary {
    pub name: String,
    pub aggregate: String,
    pub distribution: Option<String>,
    pub scale: f64,
    pub accuracy: Option<f64>,
}

/// Get noise scale parameters from a LazyFrame.
///
/// # Arguments
/// * `lazyframe` - computation from which you want to read noise scale parameters from
/// * `alpha` - optional statistical significance to use to compute accuracy estimates
pub fn lazyframe_utility(lazyframe: &LazyFrame, alpha: Option<f64>) -> Fallible<DataFrame> {
    let mut utility = logical_plan_utility(&lazyframe.logical_plan, alpha)?;

    // only include the accuracy column if alpha is passed
    if alpha.is_none() {
        utility = utility.drop("accuracy")?;
    }
    Ok(utility)
}

fn logical_plan_utility(logical_plan: &DslPlan, alpha: Option<f64>) -> Fallible<DataFrame> {
    match logical_plan {
        DslPlan::Select { expr: exprs, .. } | DslPlan::GroupBy { aggs: exprs, .. } => {
            let rows = exprs
                .iter()
                .map(|e| expr_utility(&e, alpha))
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
                        ])
                    })
                    .collect::<Vec<_>>(),
                &Schema::from_iter(vec![
                    Field::new("column", DataType::String),
                    Field::new("aggregate", DataType::String),
                    Field::new("distribution", DataType::String),
                    Field::new("scale", DataType::Float64),
                    Field::new("accuracy", DataType::Float64),
                ]),
            )?)
        }
        DslPlan::Filter { input, .. }
        | DslPlan::Sort { input, .. }
        | DslPlan::Slice { input, .. }
        | DslPlan::Sink { input, .. } => logical_plan_utility(input.as_ref(), alpha),
        dsl => fallible!(FailedFunction, "unrecognized dsl: {:?}", dsl.describe()),
    }
}

fn expr_utility<'a>(expr: &Expr, alpha: Option<f64>) -> Fallible<Vec<UtilitySummary>> {
    let name = expr.clone().meta().output_name()?.to_string();
    let expr = expr.clone().meta().undo_aliases();
    if let Some((input, plugin)) = match_noise(&expr)? {
        let scale = plugin
            .scale
            .ok_or_else(|| err!(FailedFunction, "scale must be known"))?;

        let distribution = plugin
            .distribution
            .ok_or_else(|| err!(FailedFunction, "distribution must be known"))?;

        let support = plugin
            .support
            .ok_or_else(|| err!(FailedFunction, "support must be known"))?;

        let accuracy = if let Some(alpha) = alpha {
            use {Distribution::*, Support::*};
            Some(match (distribution, support) {
                (Laplace, Float) => laplacian_scale_to_accuracy(scale, alpha),
                (Gaussian, Float) => gaussian_scale_to_accuracy(scale, alpha),
                (Laplace, Integer) => discrete_laplacian_scale_to_accuracy(scale, alpha),
                (Gaussian, Integer) => discrete_gaussian_scale_to_accuracy(scale, alpha),
            }?)
        } else {
            None
        };

        return Ok(vec![UtilitySummary {
            name,
            aggregate: expr_aggregate(input)?.to_string(),
            distribution: Some(format!("{:?} {:?}", support, distribution)),
            scale,
            accuracy,
        }]);
    }

    if let Some((input, plugin)) = match_report_noisy_max_gumbel(&expr)? {
        return Ok(vec![UtilitySummary {
            name,
            aggregate: expr_aggregate(input)?.to_string(),
            distribution: Some("Gumbel".to_string()),
            scale: plugin
                .scale
                .ok_or_else(|| err!(FailedFunction, "scale must be known"))?,
            accuracy: None,
        }]);
    }

    match expr {
        Expr::Len => Ok(vec![UtilitySummary {
            name,
            aggregate: "Len".to_string(),
            distribution: None,
            scale: 0.0,
            accuracy: alpha.is_some().then_some(0.0),
        }]),

        Expr::Function { input, .. } => Ok(input
            .iter()
            .map(|e| expr_utility(e, alpha))
            .collect::<Fallible<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect()),

        _ => fallible!(FailedFunction, "unrecognized primitive"),
    }
}

fn expr_aggregate(expr: &Expr) -> Fallible<&'static str> {
    if match_discrete_quantile_score(expr)?.is_some() {
        return Ok("Quantile");
    }
    Ok(match expr {
        Expr::Agg(AggExpr::Sum(_)) => "Sum",
        Expr::Len => "Len",
        expr => return fallible!(FailedFunction, "unrecognized aggregation: {:?}", expr),
    })
}
