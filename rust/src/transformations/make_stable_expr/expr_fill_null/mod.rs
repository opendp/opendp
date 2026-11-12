use polars::prelude::DataType;
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, ExprPlan, OuterMetric, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::MicrodataMetric;

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
) -> Fallible<Transformation<WildExprDomain, M, ExprDomain, M>>
where
    M::InnerMetric: MicrodataMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        input,
        function: FunctionExpr::FillNull { .. },
        ..
    } = &expr
    else {
        return fallible!(MakeTransformation, "expected fill_null expression");
    };

    let [data, fill] = input.as_slice() else {
        return fallible!(MakeTransformation, "fill_null expects 2 arguments");
    };

    // only enforce row-by-row context if the fill expression is not broadcastable
    let is_column_fill = !fill.clone().meta().root_names().is_empty();
    let expr_domain = if is_column_fill {
        input_domain.as_row_by_row()
    } else if let Expr::Literal(value) = fill {
        if !value.is_scalar() {
            return fallible!(MakeTransformation, "fill expression must be broadcastable");
        }
        input_domain.clone()
    } else {
        return fallible!(
            MakeTransformation,
            "fill expression must be a column or scalar"
        );
    };

    let t_data = data
        .clone()
        .make_stable(expr_domain.clone(), input_metric.clone())?;
    let t_fill = fill
        .clone()
        .make_stable(expr_domain, input_metric.clone())?;

    let (data_domain, data_metric) = t_data.output_space();
    let (fill_domain, fill_metric) = t_fill.output_space();

    if data_metric != fill_metric {
        return fallible!(
            MakeTransformation,
            "output metrics on the input and fill expressions must match: {:?} != {:?}",
            data_metric,
            fill_metric
        );
    }

    if matches!(data_domain.column.dtype(), DataType::Categorical(_, _)) {
        return fallible!(
            MakeTransformation,
            "fill_null cannot be applied to categorical data, because it may trigger a data-dependent CategoricalRemappingWarning in Polars"
        );
    }

    if fill_domain.column.nullable {
        return fallible!(MakeTransformation, "fill expression must not be nullable");
    }

    let data_dtype = data_domain.column.dtype();
    let fill_dtype = fill_domain.column.dtype();
    let is_numeric = |dtype: &DataType| dtype.is_primitive_numeric() || *dtype == DataType::Boolean;
    if data_dtype != fill_dtype && !(is_numeric(&data_dtype) && is_numeric(&fill_dtype)) {
        return fallible!(
            MakeTransformation,
            "fill_null with different input and filler types is only supported for numeric data"
        );
    }

    let can_be_nan = |column: &SeriesDomain| -> Fallible<bool> {
        Ok(match column.dtype() {
            DataType::Float32 => column.atom_domain::<f32>()?.nan(),
            DataType::Float64 => column.atom_domain::<f64>()?.nan(),
            _ => false,
        })
    };
    let output_nan = can_be_nan(&data_domain.column)?
        || (data_domain.column.nullable && can_be_nan(&fill_domain.column)?);
    let mut output_domain = data_domain.clone();
    output_domain
        .column
        .set_dtype(super::infer_output_dtype(&expr, &input_domain)?)?;
    output_domain.column.nullable = false;
    if !output_nan && output_domain.column.dtype().is_float() {
        output_domain.column.set_non_nan()?;
    }
    if is_column_fill {
        output_domain.context = input_domain.context.clone();
    }

    Transformation::new(
        input_domain,
        input_metric.clone(),
        output_domain,
        input_metric,
        Function::new_fallible(move |arg| {
            let data = t_data.invoke(arg)?;
            let fill = t_fill.invoke(arg)?;

            Ok(ExprPlan {
                plan: arg.clone(),
                expr: data.expr.fill_null(fill.expr),
                fill: data.fill.zip(fill.fill).map(|(d, f)| d.fill_null(f)),
            })
        }),
        StabilityMap::new(Clone::clone),
    )
}
