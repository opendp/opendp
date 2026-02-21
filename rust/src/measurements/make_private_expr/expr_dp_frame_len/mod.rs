#[cfg(feature = "ffi")]
use polars::prelude::CompatLevel;
#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail},
    prelude::{Column, ColumnsUdf, Expr, GetOutput, len},
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
    polars::{OpenDPPlugin, apply_plugin, match_shim},
    transformations::{StableExpr, traits::UnboundedMetric},
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct DPFrameLenShim;

impl ColumnsUdf for DPFrameLenShim {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}
impl OpenDPPlugin for DPFrameLenShim {
    const NAME: &'static str = "dp_frame_len";
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }
    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::same_type())
    }
}

pub fn make_expr_dp_frame_len<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
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
    let Some([scale]) = match_shim::<DPFrameLenShim, _>(&expr)? else {
        return fallible!(
            MakeMeasurement,
            "Expected {} function",
            DPFrameLenShim::NAME
        );
    };

    apply_plugin(vec![len(), scale], expr, NoiseShim).make_private(
        input_domain.clone(),
        input_metric,
        output_measure,
        global_scale,
    )
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type = Null)]
fn dp_frame_len(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}
