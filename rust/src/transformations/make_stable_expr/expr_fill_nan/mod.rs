use std::sync::Arc;

use polars::datatypes::DataType;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprContext, ExprDomain, OuterMetric};
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
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Some((data, fill)) = match_fill_nan(&expr) else {
        return fallible!(MakeTransformation, "expected fill_nan expression");
    };

    let ExprDomain {
        frame_domain,
        context,
    } = input_domain.clone();
    let rr_domain = ExprDomain::new(frame_domain, ExprContext::RowByRow);

    let t_data = data
        .clone()
        .make_stable(rr_domain.clone(), input_metric.clone())?;
    let t_fill = fill
        .clone()
        .make_stable(rr_domain.clone(), input_metric.clone())?;

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

    let fill_series = fill_domain.active_series()?;
    let fill_can_be_nan = match &fill_series.field.dtype {
        // from the perspective of atom domain, null refers to existence of any missing value.
        // For float types, this is NaN.
        // Therefore if the float domain may be nullable, then the domain includes NaN
        DataType::Float32 => fill_series.atom_domain::<f32>()?.nullable(),
        DataType::Float64 => fill_series.atom_domain::<f64>()?.nullable(),
        _ => return fallible!(MakeTransformation, "filler data for fill_nan must be float"),
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

    let mut output_domain = data_domain.clone();
    // fill_nan should not change the output context-- just require that its input is row-by-row
    output_domain.context = context;
    let series_domain = output_domain.active_series_mut()?;
    series_domain.drop_bounds().ok();

    series_domain.element_domain = match &series_domain.field.dtype {
        DataType::Float32 => Arc::new(AtomDomain::<f32>::default()),
        DataType::Float64 => Arc::new(AtomDomain::<f64>::default()),
        _ => {
            return fallible!(
                MakeTransformation,
                "fill_nan may only be applied to float data"
            )
        }
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg| {
            let expr_data = t_data.invoke(arg)?.1;
            let expr_fill = t_fill.invoke(arg)?.1;
            Ok((arg.0.clone(), expr_data.fill_nan(expr_fill)))
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
