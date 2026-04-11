use std::sync::Arc;

#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail, polars_err},
    prelude::{AnonymousColumnsUdf, Column, ColumnsUdf, Expr, lit},
};
use polars_plan::prelude::FunctionOptions;
use serde::{Deserialize, Serialize};

use crate::{
    core::{Measurement, MetricSpace},
    domains::{ExprDomain, ExprPlan, WildExprDomain},
    error::Fallible,
    measurements::{PrivateExpr, expr_dp_quantile::DPQuantileShim, expr_noise::NoiseExprMeasure},
    metrics::L01InfDistance,
    polars::{OpenDPPlugin, apply_plugin, match_shim},
    transformations::{StableExpr, traits::UnboundedMetric},
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct DPMedianShim;
impl ColumnsUdf for DPMedianShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Column> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl AnonymousColumnsUdf for DPMedianShim {
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

impl OpenDPPlugin for DPMedianShim {
    const NAME: &'static str = "dp_median";
    #[cfg(feature = "ffi")]
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }
}

/// Make a dp median expression measurement.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the noise will be added
/// * `global_scale` - (Re)scale the noise parameter for the noise distribution
pub fn make_expr_dp_median<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
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
    let Some([input, candidates, scale]) = match_shim::<DPMedianShim, _>(&expr)? else {
        return fallible!(MakeMeasurement, "Expected {} function", DPMedianShim::NAME);
    };

    let input = apply_plugin(
        vec![input, lit(0.5f64), candidates, scale],
        expr,
        DPQuantileShim,
    );

    input.make_private(input_domain, input_metric, output_measure, global_scale)
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn dp_median(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}
