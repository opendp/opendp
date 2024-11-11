use polars::prelude::*;
use polars_plan::dsl::Expr;
use polars_plan::utils::expr_output_name;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, OuterMetric, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a binary expression
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The clipping expression
pub fn make_expr_binary<M>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>>
where
    M: OuterMetric,
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::BinaryExpr { left, op, right } = expr.clone() else {
        return fallible!(MakeTransformation, "expected binary expression");
    };

    let t_left = left
        .as_ref()
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;
    let t_right = right
        .as_ref()
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;

    use polars_plan::dsl::Operator::*;
    if !matches!(
        op,
        Eq | NotEq | Lt | LtEq | Gt | GtEq | And | Or | Xor | LogicalAnd | LogicalOr
    ) {
        return fallible!(MakeTransformation, "unsupported operator: {:?}. Only binary operations that emit booleans are currently supported.", op);
    }

    let mut data_column =
        SeriesDomain::new(expr_output_name(&expr)?, AtomDomain::<bool>::default());

    let left_nullable = t_left.output_domain.column.nullable;
    let right_nullable = t_right.output_domain.column.nullable;

    data_column.nullable = left_nullable || right_nullable;

    let output_domain = ExprDomain {
        column: data_column,
        context: input_domain.context.clone(),
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg: &DslPlan| {
            let left = t_left.invoke(arg)?.1;
            let right = t_right.invoke(arg)?.1;

            let binary = Expr::BinaryExpr {
                left: Arc::new(left),
                right: Arc::new(right),
                op: op.clone(),
            };
            Ok((arg.clone(), binary))
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
