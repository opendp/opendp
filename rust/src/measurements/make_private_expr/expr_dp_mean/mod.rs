use std::sync::Arc;

#[cfg(feature = "ffi")]
use polars::prelude::CompatLevel;
#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail},
    prelude::{Column, ColumnsUdf, Expr, GetOutput, Operator, lit},
};
#[cfg(feature = "ffi")]
use polars_arrow as arrow;
use polars_plan::prelude::FunctionOptions;
use serde::{Deserialize, Serialize};

use crate::{
    core::{Measurement, MetricSpace},
    domains::{ExprDomain, ExprPlan, Invariant, WildExprDomain},
    error::Fallible,
    measurements::{
        PrivateExpr, expr_dp_counting_query::DPLenShim, expr_dp_sum::DPSumShim,
        expr_noise::NoiseExprMeasure,
    },
    metrics::L01InfDistance,
    polars::{OpenDPPlugin, apply_plugin, match_shim},
    transformations::{StableExpr, traits::UnboundedMetric},
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct DPMeanShim;
impl ColumnsUdf for DPMeanShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for DPMeanShim {
    const NAME: &'static str = "dp_mean";
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::same_type())
    }
}

/// Make a dp sum expression measurement.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the noise will be added
/// * `global_scale` - (Re)scale the noise parameter for the noise distribution
pub fn make_expr_dp_mean<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    output_measure: MO,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, L01InfDistance<MI>, MO, ExprPlan>>
where
    Expr: StableExpr<L01InfDistance<MI>, L01InfDistance<MI>> + PrivateExpr<L01InfDistance<MI>, MO>,
    (ExprDomain, MO::Metric): MetricSpace,
{
    let Some([input, lower, upper, scale]) = match_shim::<DPMeanShim, _>(&expr)? else {
        return fallible!(MakeMeasurement, "Expected {} function", DPMeanShim::NAME);
    };

    let t_prior = input
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;

    let scale_denom = match t_prior.output_domain.context.aggregation("mean")?.invariant {
        Some(Invariant::Lengths) => lit(0f64),
        // balances the variance between the numerator and denominator,
        // which (under the assumption that data is on average halfway between the bounds)
        // maximizes utility
        _ => scale.clone(),
    };

    let sum = apply_plugin(
        vec![input.clone(), lower, upper, scale],
        expr.clone(),
        DPSumShim,
    );
    let len = apply_plugin(vec![input, scale_denom], expr, DPLenShim);

    Expr::BinaryExpr {
        left: Arc::new(sum),
        op: Operator::TrueDivide,
        right: Arc::new(len),
    }
    .make_private(input_domain, input_metric, output_measure, global_scale)
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn dp_mean(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}
