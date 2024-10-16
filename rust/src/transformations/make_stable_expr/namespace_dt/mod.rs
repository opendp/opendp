use expr_datetime_component::{make_expr_datetime_component, match_datetime_component};
use polars::prelude::FunctionExpr;
use polars_plan::dsl::Expr;

use crate::core::{MetricSpace, Transformation};
use crate::domains::{ExprDomain, OuterMetric};
use crate::error::*;
use crate::polars::get_disabled_features_message;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(feature = "contrib")]
mod expr_datetime_component;

#[cfg(test)]
pub(crate) mod test;

/// Make a Transformation that returns an expression under the `dt` namespace.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The dt expression
pub fn make_namespace_dt<M: OuterMetric>(
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        function: FunctionExpr::TemporalExpr(temporal_function),
        ..
    } = &expr
    else {
        return fallible!(MakeTransformation, "expected temporal expression");
    };

    #[cfg(feature = "contrib")]
    if match_datetime_component(temporal_function.clone()).is_some() {
        return make_expr_datetime_component(input_domain, input_metric, expr);
    }

    fallible!(
        MakeTransformation,
        "Expr is not recognized at this time: {:?}. {}If you would like to see this supported, please file an issue.",
        expr,
        get_disabled_features_message()
    )
}
