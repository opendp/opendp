use polars::prelude::*;
use polars_plan::dsl::{BooleanFunction, Expr, FunctionExpr};
use polars_plan::prelude::{ApplyOptions, FunctionOptions};

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, OuterMetric};
use crate::error::*;
use crate::polars::ExprFunction;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a boolean function expression
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The boolean function expression
pub fn make_expr_boolean_function<M: OuterMetric>(
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
        function: FunctionExpr::Boolean(bool_function),
        ..
    } = expr
    else {
        return fallible!(MakeTransformation, "expected boolean function expression");
    };

    use BooleanFunction::*;

    if matches!(bool_function, Any { .. } | All { .. }) {
        return fallible!(
            MakeTransformation,
            "{:?} will not be supported, as this aggregation is too sensitive to extreme values to be estimated with reasonable utility",
            bool_function
        );
    }

    if !matches!(
        bool_function,
        IsNull | IsNotNull | IsFinite | IsInfinite | IsNan | IsNotNan | Not
    ) {
        return fallible!(
            MakeTransformation,
            "{:?} is not currently supported",
            bool_function
        );
    }

    let Ok([input]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeTransformation,
            "{} must have one argument, found {}",
            bool_function,
            input.len()
        );
    };

    let t_prior = input
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let active_series = output_domain.active_series_mut()?;

    if matches!(bool_function, IsNull | IsNotNull) {
        active_series.nullable = false;
    }

    if matches!(bool_function, Not) && active_series.field.dtype != DataType::Boolean {
        // under these conditions, the expression performs a bitwise negation
        active_series.drop_bounds()?;
    } else {
        // otherwise, the output data will be a bool
        active_series.element_domain = Arc::new(AtomDomain::<bool>::default());
        active_series.field.dtype = DataType::Boolean;
    }

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            output_domain,
            Function::then_expr(move |expr| Expr::Function {
                input: vec![expr],
                function: FunctionExpr::Boolean(bool_function.clone()),
                options: FunctionOptions {
                    collect_groups: ApplyOptions::ElementWise,
                    ..Default::default()
                },
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
