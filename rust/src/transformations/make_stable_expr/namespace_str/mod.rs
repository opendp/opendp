use polars::prelude::{FunctionExpr, StringFunction};
use polars_plan::dsl::Expr;

use crate::core::{MetricSpace, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::polars::get_disabled_features_message;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(feature = "contrib")]
mod expr_strptime;

/// Make a Transformation that returns an expression under the `str` namespace.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The str expression
pub fn make_namespace_str<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        function: FunctionExpr::StringExpr(str_function),
        ..
    } = &expr
    else {
        return fallible!(MakeTransformation, "expected cast expression");
    };

    match str_function {
        #[cfg(feature = "contrib")]
        StringFunction::Strptime(_, _) => expr_strptime::make_expr_strptime(input_domain, input_metric, expr),

        expr => fallible!(
            MakeTransformation,
            "Expr is not recognized at this time: {:?}. {}If you would like to see this supported, please file an issue.",
            expr,
            get_disabled_features_message()
        )
    }
}
