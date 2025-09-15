#[cfg(feature = "ffi")]
use polars::prelude::CompatLevel;
#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail},
    prelude::{Column, ColumnsUdf, DataType, Expr, GetOutput, Literal, LiteralValue, lit},
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
        PrivateExpr,
        expr_noise::{NoiseExprMeasure, NoiseShim},
    },
    metrics::L01InfDistance,
    polars::{ExtractValue, OpenDPPlugin, apply_plugin, literal_value_of, match_shim},
    traits::Number,
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

            fn get_midpoint<T: Number + ExtractValue + Literal>(
                lower: &Expr,
                upper: &Expr,
            ) -> Option<Expr> {
                let lower = literal_value_of::<T>(&lower).ok()??;
                let upper = literal_value_of::<T>(&upper).ok()??;
                Some(lit((lower + upper) / (T::one() + T::one())))
            }

            fn get_filler<T: Number + ExtractValue + Literal>(lower: &Expr, upper: &Expr) -> Expr {
                get_midpoint::<T>(lower, upper).unwrap_or_else(|| lower.clone())
            }

            let filler = match series_domain.dtype() {
                DataType::UInt8 => get_filler::<u8>(&lower, &upper),
                DataType::UInt16 => get_filler::<u16>(&lower, &upper),
                DataType::UInt32 => get_filler::<u32>(&lower, &upper),
                DataType::UInt64 => get_filler::<u64>(&lower, &upper),
                DataType::Int8 => get_filler::<i8>(&lower, &upper),
                DataType::Int16 => get_filler::<i16>(&lower, &upper),
                DataType::Int32 => get_filler::<i32>(&lower, &upper),
                DataType::Int64 => get_filler::<i64>(&lower, &upper),
                DataType::Float32 => get_filler::<f32>(&lower, &upper),
                DataType::Float64 => get_filler::<f64>(&lower, &upper),
                _ => return fallible!(MakeMeasurement, "DP Sum input must be numeric"),
            };

            input = input.fill_null(filler.clone());

            if series_domain.dtype().is_float() {
                input = input.fill_nan(filler)
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

    apply_plugin(vec![input.sum(), scale], expr, NoiseShim).make_private(
        input_domain,
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
