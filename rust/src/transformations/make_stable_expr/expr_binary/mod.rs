use polars::prelude::*;
use polars_plan::dsl::Expr;
use polars_plan::utils::expr_output_name;
// This code is not particular to Python, so we shouldn't need the higher-level library here.
// https://github.com/opendp/opendp/issues/2309
use pyo3_polars::export::polars_core::utils::try_get_supertype;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, ExprPlan, OuterMetric, SeriesDomain, WildExprDomain};
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

    let data_column = match op {
        Eq | NotEq | Lt | LtEq | Gt | GtEq | And | Or | Xor | LogicalAnd | LogicalOr => {
            let mut series_domain =
                SeriesDomain::new(expr_output_name(&expr)?, AtomDomain::<bool>::default());
            series_domain.nullable = left_series.nullable || right_series.nullable;
            series_domain
        }
        Plus | Minus | Multiply | Divide | TrueDivide | FloorDivide => {
            let common_dtype = try_get_supertype(&left_series.dtype(), &right_series.dtype())?;

            let out_dtype = match op {
                Plus | Minus | Multiply | FloorDivide => common_dtype,
                TrueDivide | Divide => {
                    if common_dtype.is_float() {
                        common_dtype
                    } else {
                        DataType::Float64
                    }
                }
                _ => unreachable!("due to above match arm"),
            };

            let mut series_domain = SeriesDomain::new_from_field(Field::new(
                left_series.name.clone(),
                out_dtype.clone(),
            ))?;

            // output is nullable when op is FloorDivide since a // 0 is null for any a
            series_domain.nullable =
                left_series.nullable || right_series.nullable || matches!(op, FloorDivide);

            series_domain
        }
        _ => {
            return fallible!(
                MakeTransformation,
                "unsupported operator: {:?}. Only arithmetic operations or binary operations that emit booleans are currently supported.",
                op
            );
        }
    };

    let output_domain = ExprDomain {
        column: data_column,
        context: input_domain.context.clone(),
    };

    Transformation::new(
        input_domain,
        output_domain,
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
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
