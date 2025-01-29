use polars::datatypes::DataType;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, ExprPlan, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `fill_nan` expression
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The fill_nan expression
pub fn make_expr_fill_nan<M: OuterMetric>(
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
    let Some((data, fill)) = match_fill_nan(&expr) else {
        return fallible!(MakeTransformation, "expected fill_nan expression");
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
            "interior metrics on the input and fill expressions must match: {:?} != {:?}",
            data_metric,
            fill_metric
        );
    }

    let fill_series = &fill_domain.column;
    let fill_can_be_nan = match fill_series.dtype() {
        // from the perspective of atom domain, null refers to existence of any missing value.
        // For float types, this is NaN.
        // Therefore if the float domain may be nullable, then the domain includes NaN
        DataType::Float32 => fill_series.atom_domain::<f32>()?.nullable(),
        DataType::Float64 => fill_series.atom_domain::<f64>()?.nullable(),
        i if i.is_numeric() => false,
        _ => {
            return fallible!(
                MakeTransformation,
                "filler data for fill_nan must be numeric"
            )
        }
    };

    if fill_can_be_nan {
        return fallible!(
            MakeTransformation,
            "filler data for fill_nan must not contain nan"
        );
    }
    if fill_series.nullable {
        return fallible!(
            MakeTransformation,
            "filler data for fill_nan must not be nullable"
        );
    }

    let mut series_domain = data_domain.column.clone();
    match series_domain.dtype() {
        DataType::Float32 => series_domain.set_element_domain(AtomDomain::<f32>::new(None, None)),
        DataType::Float64 => series_domain.set_element_domain(AtomDomain::<f64>::new(None, None)),
        _ => {
            return fallible!(
                MakeTransformation,
                "fill_nan may only be applied to float data"
            )
        }
    }
    let output_domain = ExprDomain {
        column: series_domain,
        // fill_nan should not change the output context-- just require that its input is row-by-row
        context: input_domain.context.clone(),
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg| {
            let data = t_data.invoke(arg)?;
            let fill = t_fill.invoke(arg)?;

            Ok(ExprPlan {
                plan: arg.clone(),
                expr: data.expr.fill_nan(fill.expr),
                fill: data.fill.zip(fill.fill).map(|(d, f)| d.fill_nan(f)),
            })
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}

/// If the passed expression is fill_null (a ternary conditioned on data is_not_nan),
/// then returns the data and fill expressions.
pub fn match_fill_nan(expr: &Expr) -> Option<(&Expr, &Expr)> {
    let Expr::Ternary {
        predicate,
        truthy,
        falsy,
    } = expr
    else {
        return None;
    };

    let expected_predicate = truthy
        .as_ref()
        .clone()
        .is_not_nan()
        .or(truthy.as_ref().clone().is_null());

    if predicate.as_ref() != &expected_predicate {
        return None;
    }

    Some((truthy.as_ref(), falsy.as_ref()))
}
