use std::sync::Arc;

use polars::error::polars_err;
use polars::prelude::AnonymousColumnsUdf;
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail},
    prelude::{Column, ColumnsUdf, Expr, lit},
};
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
        StableExpr,
        expr_discrete_quantile_score::{DiscreteQuantileScoreShim, cast_quantile_candidates},
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

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Column> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl AnonymousColumnsUdf for DPQuantileShim {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }

    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }

    fn get_field(
        &self,
        _: &polars::prelude::Schema,
        fields: &[polars::prelude::Field],
    ) -> PolarsResult<polars::prelude::Field> {
        <&[polars::prelude::Field; 1]>::try_from(fields)
            .map_err(|_| polars_err!(InvalidOperation: "{} expects one column", Self::NAME))
            .map(|[x]| x.clone())
    }
}

impl OpenDPPlugin for DPQuantileShim {
    const NAME: &'static str = "dp_quantile";
    #[cfg(feature = "ffi")]
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }
}

/// Make a dp quantile expression measurement.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the noise will be added
/// * `global_scale` - (Re)scale the noise parameter for the noise distribution
pub(crate) fn make_expr_dp_quantile<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
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

    let candidates_series = literal_value_of::<Series>(&candidates)?
        .ok_or_else(|| err!(MakeMeasurement, "candidates must be known"))?;
    if candidates_series.is_empty() {
        return fallible!(MakeMeasurement, "candidates must be non-empty");
    }
    // Score and release the same values; never round private data through a candidate dtype.
    let candidates_typed = cast_quantile_candidates(&candidates_series, &series_domain.dtype())?;
    // An f64 median would promote integer inputs to Float64 before scoring.
    let fill_value = Expr::Literal(candidates_typed.get(candidates_typed.len() / 2)?.into());
    let candidates = lit(candidates_typed);
    input = input.fill_null(fill_value.clone());

    if series_domain.dtype().is_float() {
        input = input.fill_nan(fill_value)
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
