use std::sync::Arc;

#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail, polars_err},
    prelude::{AnonymousColumnsUdf, Column, ColumnsUdf, DataType, Expr, Field},
};
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

            fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Column> {
                polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
            }
        }

        impl AnonymousColumnsUdf for $plugin {
            fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
                self
            }

            fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
                Arc::new(Arc::unwrap_or_clone(self))
            }

            fn get_field(&self, _: &polars::prelude::Schema, fields: &[polars::prelude::Field]) -> PolarsResult<polars::prelude::Field> {
                let [input, _scale, _signed] = <&[Field; 3]>::try_from(fields)
                    .map_err(|_| polars_err!(InvalidOperation: "{} expects three arguments", Self::NAME))?;

                Ok(Field::new(input.name().clone(), DataType::UInt32))
            }
        }

        impl OpenDPPlugin for $plugin {
            const NAME: &'static str = $name;
            #[cfg(feature = "ffi")]
            const SHIM: bool = true;
            fn function_options() -> FunctionOptions {
                FunctionOptions::aggregation()
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
            let Some([input, scale, signed]) = match_shim::<$plugin, 3>(&expr)? else {
                return fallible!(MakeMeasurement, "Expected {} function", $plugin::NAME);
            };

            let signed = match signed {
                Expr::Literal(lit) => lit.bool().unwrap_or(false),
                _ => return fallible!(MakeMeasurement, "signed argument must be a literal bool"),
            };

            let aggregate = input.$stable_method();
            let aggregate = if signed {
                aggregate.cast(DataType::Int64)
            } else {
                aggregate
            };

            apply_plugin(vec![aggregate, scale], expr, NoiseShim).make_private(
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

#[cfg(test)]
mod test;

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
