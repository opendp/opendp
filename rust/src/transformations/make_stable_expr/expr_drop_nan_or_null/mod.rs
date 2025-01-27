use polars::prelude::FunctionExpr;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, Margin, MarginPub, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::transformations::traits::UnboundedMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `drop_nulls` expression or `drop_nans` expression.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The drop_nulls or drop_nans expression
pub fn make_expr_drop_nan_or_null<M: OuterMetric>(
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
    let Expr::Function {
        input,
        function,
        options,
    } = expr
    else {
        return fallible!(MakeTransformation, "expected function expression");
    };

    let op_name = function.to_string();

    let margin = input_domain.context.aggregation(op_name.as_str())?;

    let [input] = <[Expr; 1]>::try_from(input)
        .map_err(|_| err!(MakeTransformation, "{} takes one input", op_name.as_str()))?;

    let t_input = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_input.output_space();

    let mut series_domain = t_input.output_domain.column.clone();
    match function {
        FunctionExpr::DropNans => series_domain.set_non_nan()?,
        FunctionExpr::DropNulls => series_domain.nullable = false,
        _ => {
            return fallible!(
                MakeTransformation,
                "expected drop_nans or drop_nulls expression"
            )
        }
    }

    // Even if all records are filtered out, the partition remains.
    // Therefore the margin key descriptor is preserved.
    // However, the partition length is not preserved.
    let output_domain = ExprDomain {
        column: series_domain,
        context: Context::Aggregation {
            margin: Margin {
                public_info: margin.public_info.map(|_| MarginPub::Keys),
                ..margin
            },
        },
    };

    t_input
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| Expr::Function {
                input: vec![expr],
                function: function.clone(),
                options: options.clone(),
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
