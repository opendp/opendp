use polars::prelude::*;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric, WildExprDomain};
use crate::error::Fallible;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `col(column_name)` expression for a Lazy Frame.
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
/// * `expr` - A column expression.
pub fn make_expr_col<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>>
where
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
{
    let Expr::Column(col_name) = expr else {
        return fallible!(MakeTransformation, "Expected col() expression");
    };

    let output_domain = ExprDomain {
        column: (input_domain.columns.iter())
            .find(|s| s.name == col_name)
            .ok_or_else(|| err!(MakeTransformation, "unrecognized column: {}", col_name))?
            .clone(),
        context: input_domain.context.clone(),
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::from_expr(col(&*col_name)),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
