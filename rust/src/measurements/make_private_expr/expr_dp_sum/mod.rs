#[cfg(feature = "ffi")]
use polars::prelude::CompatLevel;
#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail},
    prelude::{Column, ColumnsUdf, DataType, Expr, GetOutput, LiteralValue},
};
#[cfg(feature = "ffi")]
use polars_arrow as arrow;
use polars_plan::prelude::FunctionOptions;
use serde::{Deserialize, Serialize};

use crate::{
    core::{Measurement, MetricSpace},
    domains::{ExprDomain, ExprPlan, WildExprDomain},
    error::Fallible,
    measurements::{PrivateExpr, expr_noise::NoiseExprMeasure},
    metrics::L01InfDistance,
    polars::{OpenDPPlugin, PrivacyNamespace, literal_value_of, match_shim},
    transformations::{StableExpr, traits::UnboundedMetric},
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct DPSumShim;
impl ColumnsUdf for DPSumShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for DPSumShim {
    const NAME: &'static str = "dp_sum";
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
pub fn make_expr_dp_sum<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
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
    let Some([mut input, lower, upper, scale]) = match_shim::<DPSumShim, _>(&expr)? else {
        return fallible!(MakeMeasurement, "Expected {} function", DPSumShim::NAME);
    };
    let scale = literal_value_of::<f64>(&scale)?;

    fn is_null(expr: &Expr) -> bool {
        let Expr::Literal(LiteralValue::Scalar(scalar)) = expr else {
            return false;
        };
        scalar.dtype() == &DataType::Null
    }

    match (is_null(&lower), is_null(&upper)) {
        (false, false) => {
            let t_prior = input
                .clone()
                .make_stable(input_domain.clone(), input_metric.clone())?;
            let series_domain = t_prior.output_domain.column.clone();

            let midpoint = lower.clone();
            input = input.fill_null(midpoint.clone());

            if series_domain.dtype().is_float() {
                input = input.fill_nan(midpoint)
            }

            input = input.clip(lower, upper)
        }
        (true, true) => (),
        _ => {
            return fallible!(
                MakeMeasurement,
                "dp_sum: bounds ({:?}, {:?}) must both be specified",
                lower,
                upper
            );
        }
    };

    input = input.sum().dp().noise(scale);

    input.make_private(
        input_domain.clone(),
        input_metric,
        output_measure,
        global_scale,
    )
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn dp_sum(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}
