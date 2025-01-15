use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that renames a column in a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The alias expression
pub fn make_expr_alias<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>>
where
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Alias(input, name) = expr else {
        return fallible!(MakeTransformation, "expected alias expression");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    output_domain.column.name = name.clone();

    let t_alias = Transformation::new(
        middle_domain.clone(),
        output_domain,
        Function::then_expr(move |expr| expr.alias(name.clone())),
        middle_metric.clone(),
        middle_metric,
        StabilityMap::new(Clone::clone),
    )?;

    t_prior >> t_alias
}
