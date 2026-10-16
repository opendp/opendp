use polars::prelude::{Column, DataFrame, DataType, IntoLazy};
use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{
    AtomDomain, ExprDomain, ExprPlan, NaN, OuterMetric, SeriesDomain, WildExprDomain,
};
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
    } = expr
    else {
        return fallible!(MakeTransformation, "expected fill_null expression");
    };

    let Ok([data, fill]) = <[_; 2]>::try_from(input) else {
        return fallible!(MakeTransformation, "fill_null expects 2 arguments");
    };

    // only enforce row-by-row context if the fill expression is not broadcastable
    let expr_domain = if fill.clone().meta().root_names().len() > 0 {
        input_domain.as_row_by_row()
    } else if let Expr::Literal(value) = fill.clone() {
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

    // Infer the result type from the stabilized expressions, preserving Polars'
    // coercion rules for dynamic literals and mixed numeric inputs. Only public
    // schema information is used; no private data is evaluated.
    let mock = DataFrame::new(
        input_domain
            .columns
            .iter()
            .map(|column| Column::new_empty(column.name.clone(), &column.dtype()))
            .collect(),
    )?
    .lazy();
    let mock_data = t_data.invoke(&mock.logical_plan)?;
    let mock_fill = t_fill.invoke(&mock.logical_plan)?;
    let schema = mock
        .select([mock_data.expr.fill_null(mock_fill.expr)])
        .collect_schema()?;
    let output_dtype = schema
        .get(&data_domain.column.name)
        .ok_or_else(|| err!(MakeTransformation, "fill_null output column is missing"))?
        .clone();

    let is_numeric = |dtype: DataType| dtype.is_primitive_numeric() || dtype == DataType::Boolean;
    if data_domain.column.dtype() != fill_domain.column.dtype()
        && !(is_numeric(data_domain.column.dtype()) && is_numeric(fill_domain.column.dtype()))
    {
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
    // A non-null input never selects the filler, even when it affects coercion.
    let output_nan = can_be_nan(&data_domain.column)?
        || (data_domain.column.nullable && can_be_nan(&fill_domain.column)?);

    let mut output_domain = data_domain.clone();
    match output_dtype {
        DataType::Float32 => output_domain
            .column
            .set_element_domain(AtomDomain::<f32>::new(None, output_nan.then(NaN::new))),
        DataType::Float64 => output_domain
            .column
            .set_element_domain(AtomDomain::<f64>::new(None, output_nan.then(NaN::new))),
        dtype if dtype != output_domain.column.dtype() => output_domain.column.set_dtype(dtype)?,
        _ => {
            output_domain.column.drop_bounds().ok();
        }
    }
    output_domain.column.nullable = false;
    output_domain.context = input_domain.context.clone();

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
