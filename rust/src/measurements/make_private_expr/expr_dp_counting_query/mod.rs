#[cfg(feature = "ffi")]
use polars::prelude::CompatLevel;
#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail},
    prelude::{Column, ColumnsUdf, Expr, GetOutput},
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

macro_rules! new_make_expr_counting_query {
    ($plugin:ident, $name:literal, $stable_method:ident, $dp_method:ident, $constructor:ident) => {
        #[derive(Clone, Serialize, Deserialize)]
        pub(crate) struct $plugin;
        impl ColumnsUdf for $plugin {
            // makes it possible to downcast the AnonymousFunction trait object back to Self
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
                polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
            }
        }

        impl OpenDPPlugin for $plugin {
            const NAME: &'static str = $name;
            const SHIM: bool = true;
            fn function_options() -> FunctionOptions {
                FunctionOptions::aggregation()
            }

            fn get_output(&self) -> Option<GetOutput> {
                Some(GetOutput::same_type())
            }
        }

        pub fn $constructor<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
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
            let Some([input, scale]) = match_shim::<$plugin, _>(&expr)? else {
                return fallible!(MakeMeasurement, "Expected {} function", $plugin::NAME);
            };

            apply_plugin(vec![input.$stable_method(), scale], expr, NoiseShim).make_private(
                input_domain.clone(),
                input_metric,
                output_measure,
                global_scale,
            )
        }

        #[cfg(feature = "ffi")]
        #[pyo3_polars::derive::polars_expr(output_type=Null)]
        fn $dp_method(_: &[Series]) -> PolarsResult<Series> {
            polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
        }

    }
}

new_make_expr_counting_query!(DPLenShim, "dp_len", len, dp_len, make_expr_dp_len);
new_make_expr_counting_query!(DPCountShim, "dp_count", count, dp_count, make_expr_dp_count);
new_make_expr_counting_query!(
    DPNullCountShim,
    "dp_null_count",
    null_count,
    dp_null_count,
    make_expr_dp_null_count
);
new_make_expr_counting_query!(
    DPNUniqueShim,
    "dp_n_unique",
    n_unique,
    dp_n_unique,
    make_expr_dp_n_unique
);
