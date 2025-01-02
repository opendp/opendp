use polars::prelude::DataType;
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, ExprPlan, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `fill_null` expression
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The fill_null expression
pub fn make_expr_fill_null<M: OuterMetric>(
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
        input,
        function: FunctionExpr::FillNull { .. },
        ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected fill_null expression");
    };

    let Ok([data, fill]) = <[_; 2]>::try_from(input) else {
        return fallible!(MakeTransformation, "fill_null expects 2 arguments");
    };

    let t_data = data
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;
    let t_fill = fill
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;

    let (data_domain, data_metric) = t_data.output_space();
    let (fill_domain, fill_metric) = t_fill.output_space();

    if data_metric != fill_metric {
        return fallible!(
            MakeTransformation,
            "interior metrics on the input and fill expressions must match: {:?} != {:?}",
            data_metric,
            fill_metric
        );
    }

    if matches!(data_domain.column.dtype(), DataType::Categorical(_, _)) {
        return fallible!(MakeTransformation, "fill_null cannot be applied to categorical data, because it may trigger a data-dependent CategoricalRemappingWarning in Polars");
    }

    if fill_domain.column.nullable {
        return fallible!(MakeTransformation, "fill expression must not be nullable");
    }

    let mut output_domain = data_domain.clone();
    output_domain.column.drop_bounds().ok();
    output_domain.column.nullable = false;
    output_domain.context = input_domain.context.clone();

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg| {
            let data = t_data.invoke(arg)?;
            let fill = t_fill.invoke(arg)?;

            Ok(ExprPlan {
                plan: arg.clone(),
                expr: data.expr.fill_null(fill.expr),
                fill: data.fill.zip(fill.fill).map(|(d, f)| d.fill_null(f)),
            })
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
