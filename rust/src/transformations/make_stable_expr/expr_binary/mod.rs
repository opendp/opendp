use polars::prelude::*;
use polars_plan::dsl::Expr;
use polars_plan::utils::expr_output_name;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, ExprPlan, OuterMetric, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::MicrodataMetric;

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
) -> Fallible<Transformation<WildExprDomain, M, ExprDomain, M>>
where
    M: OuterMetric,
    M::InnerMetric: MicrodataMetric,
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

    let left_series = &t_left.output_domain.column;
    let right_series = &t_right.output_domain.column;

    if matches!(left_series.dtype(), DataType::Categorical(_, _))
        || matches!(right_series.dtype(), DataType::Categorical(_, _))
    {
        return fallible!(
            MakeTransformation,
            "{} cannot be applied to categorical data, because it may trigger a data-dependent CategoricalRemappingWarning in Polars",
            op
        );
    }

    use polars_plan::dsl::Operator::*;

    if !matches!(
        op,
        Eq | EqValidity
            | NotEq
            | NotEqValidity
            | Lt
            | LtEq
            | Gt
            | GtEq
            | Plus
            | Minus
            | Multiply
            | Divide
            | TrueDivide
            | FloorDivide
            | Modulus
            | And
            | Or
            | Xor
            | LogicalAnd
            | LogicalOr
    ) {
        return fallible!(
            MakeTransformation,
            "unsupported binary operator: {:?}. Please open an issue on the OpenDP GitHub if you would like to see this supported.",
            op
        );
    }
    // Comparing Enum with private strings can fail when a string is not a category.
    // That error depends on the data, so reject these ordering comparisons.
    if matches!(op, Lt | LtEq | Gt | GtEq)
        && matches!(
            (left_series.dtype(), right_series.dtype()),
            (DataType::Enum(_, _), DataType::String) | (DataType::String, DataType::Enum(_, _))
        )
    {
        return fallible!(
            MakeTransformation,
            "ordering comparisons between Enum and String are not stable"
        );
    }

    let out_dtype = super::infer_output_dtype(&expr, &input_domain)?;

    let field = Field::new(expr_output_name(&expr)?, out_dtype.clone());
    let mut series_domain = SeriesDomain::new_from_field(field)?;

    // division or remainder by zero may introduce null values
    series_domain.nullable = left_series.nullable
        || right_series.nullable
        || matches!(op, FloorDivide | TrueDivide | Divide | Modulus);

    // these ops eliminate all nulls, regardless of the input
    if matches!(op, EqValidity | NotEqValidity) {
        series_domain.nullable = false;
    }

    let output_domain = ExprDomain {
        column: series_domain,
        context: input_domain.context.clone(),
    };

    Transformation::new(
        input_domain,
        input_metric.clone(),
        output_domain,
        input_metric,
        Function::new_fallible(move |arg: &DslPlan| {
            let left = t_left.invoke(arg)?;
            let right = t_right.invoke(arg)?;

            Ok(ExprPlan {
                plan: arg.clone(),
                expr: Expr::BinaryExpr {
                    left: Arc::new(left.expr),
                    right: Arc::new(right.expr),
                    op: op.clone(),
                },
                // Since this is None, if binary expressions are used after the aggregation,
                // execution will fail in a data-independent way.
                // But you can't use binary ops after aggs anyways, so this failure is unreachable.
                fill: None,
            })
        }),
        StabilityMap::new(Clone::clone),
    )
}
