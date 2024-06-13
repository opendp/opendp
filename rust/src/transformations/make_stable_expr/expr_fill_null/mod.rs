use polars_plan::dsl::{Expr, FunctionExpr};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprContext, ExprDomain, OuterMetric};
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

    if fill_domain.active_series()?.nullable {
        return fallible!(MakeTransformation, "fill expression must not be nullable");
    }

    let mut output_domain = data_domain.clone();
    // fill_null should not change the output context-- just require that its input is row-by-row
    output_domain.context = context;

    let series_domain = output_domain.active_series_mut()?;
    series_domain.drop_bounds().ok();
    series_domain.nullable = false;

    let dtype = series_domain.field.dtype.clone();

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg| {
            let expr_data = t_data.invoke(arg)?.1;
            let expr_fill = t_fill.invoke(arg)?.1;

            let mut expr_impute = expr_data.fill_null(expr_fill);

            // Update the super type of the fill_null function
            // This is necessary because it initializes to Unknown,
            // and Unknown dtype panics when serialized for FFI.
            let Expr::Function {
                function: FunctionExpr::FillNull { super_type },
                ..
            } = &mut expr_impute
            else {
                unreachable!();
            };
            *super_type = dtype.clone();
            Ok((arg.0.clone(), expr_impute))
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
