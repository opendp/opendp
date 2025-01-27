use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    Context, ExprDomain, ExprPlan, Margin, MarginPub, OuterMetric, WildExprDomain,
};
use crate::error::*;
use crate::transformations::traits::UnboundedMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `filter(predicate)` expression.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The filter expression
pub fn make_expr_filter<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: UnboundedMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Filter { input, by } = expr else {
        return fallible!(MakeTransformation, "expected filter expression");
    };

    let margin = input_domain.context.aggregation("filter")?;

    let t_input = input
        .as_ref()
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;
    let t_by = by
        .as_ref()
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;

    if t_input.output_metric != t_by.output_metric {
        return fallible!(
            MakeTransformation,
            "output metrics on the input and by expressions must match: {:?} != {:?}",
            t_input.output_metric,
            t_by.output_metric
        );
    }

    let pred_dtype = t_by.output_domain.column.dtype();

    if !pred_dtype.is_bool() {
        return fallible!(
            MakeTransformation,
            "Expected predicate to return a boolean value, got: {:?}",
            pred_dtype
        );
    }

    // Even if all records are filtered out, the partition remains.
    // Therefore the margin key descriptor is preserved.
    // However, the partition length is not preserved.
    let output_domain = ExprDomain {
        column: t_input.output_domain.column.clone(),
        context: Context::Aggregation {
            margin: Margin {
                public_info: margin.public_info.map(|_| MarginPub::Keys),
                ..margin
            },
        },
    };

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(move |arg| {
            let input = t_input.invoke(arg)?;
            let by = t_by.invoke(arg)?;

            Ok(ExprPlan {
                plan: arg.clone(),
                expr: input.expr.filter(by.expr),
                fill: None,
            })
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
