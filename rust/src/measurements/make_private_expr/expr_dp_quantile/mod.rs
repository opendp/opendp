#[cfg(feature = "ffi")]
use polars::prelude::CompatLevel;
#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail},
    prelude::{Column, ColumnsUdf, Expr, GetOutput, lit},
};
#[cfg(feature = "ffi")]
use polars_arrow as arrow;
use polars_plan::prelude::FunctionOptions;
use serde::{Deserialize, Serialize};

use crate::{
    core::{Measurement, MetricSpace},
    domains::{ExprDomain, ExprPlan, WildExprDomain},
    error::Fallible,
    measurements::{
        PrivateExpr, expr_index_candidates::IndexCandidatesShim, expr_noise::NoiseExprMeasure,
        expr_noisy_max::NoisyMaxShim,
    },
    metrics::L01InfDistance,
    polars::{OpenDPPlugin, apply_anonymous_function, literal_value_of, match_shim},
    transformations::{
        StableExpr, expr_discrete_quantile_score::DiscreteQuantileScoreShim,
        traits::UnboundedMetric,
    },
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct DPQuantileShim;
impl ColumnsUdf for DPQuantileShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for DPQuantileShim {
    const NAME: &'static str = "dp_quantile";
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::same_type())
    }
}

/// Make a dp quantile expression measurement.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the noise will be added
/// * `global_scale` - (Re)scale the noise parameter for the noise distribution
pub fn make_expr_dp_quantile<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
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
    let Some([mut input, alpha, candidates, scale]) = match_shim::<DPQuantileShim, _>(&expr)?
    else {
        return fallible!(
            MakeMeasurement,
            "Expected {} function",
            DPQuantileShim::NAME
        );
    };

    let t_prior = input
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let series_domain = t_prior.output_domain.column.clone();

    let midpoint = literal_value_of::<Series>(&candidates)?
        .and_then(|s| s.median())
        .ok_or_else(|| err!(MakeMeasurement, "candidates must be non-empty"))?;
    input = input.fill_null(lit(midpoint));

    if series_domain.dtype().is_float() {
        input = input.fill_nan(lit(midpoint))
    }
    input = apply_anonymous_function(
        vec![input, alpha, candidates.clone()],
        DiscreteQuantileScoreShim,
    );

    let negate = lit(true);
    input = apply_anonymous_function(vec![input, negate, scale], NoisyMaxShim);
    input = apply_anonymous_function(vec![input, candidates], IndexCandidatesShim);

    input.make_private(input_domain, input_metric, output_measure, global_scale)
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn dp_quantile(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}
