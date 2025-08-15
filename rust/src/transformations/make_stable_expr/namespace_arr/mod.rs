use polars::prelude::FunctionExpr;
use polars_plan::dsl::Expr;

use crate::core::{MetricSpace, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::metrics::MicrodataMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns an expression under the `arr` namespace.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The arr expression
pub fn make_namespace_arr<M: OuterMetric>(
    _input_domain: WildExprDomain,
    _input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, M, ExprDomain, M>>
where
    M::InnerMetric: MicrodataMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        function: FunctionExpr::ArrayExpr(array_function),
        ..
    } = &expr
    else {
        return fallible!(MakeTransformation, "expected array expression");
    };

    fallible!(
        MakeTransformation,
        "Expr is not recognized at this time: {:?}. Waiting for: https://github.com/pola-rs/polars/pull/20421",
        array_function
    )
}
