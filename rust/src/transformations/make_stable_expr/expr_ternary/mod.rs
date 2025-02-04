use polars::prelude::*;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, ExprPlan, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `ternary` expression
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The ternary expression
pub fn make_expr_ternary<M: OuterMetric>(
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
    let Expr::Ternary {
        predicate,
        truthy,
        falsy,
    } = expr
    else {
        return fallible!(MakeTransformation, "expected ternary expression");
    };

    let t_predicate = predicate
        .as_ref()
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;
    let t_truthy = truthy
        .as_ref()
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;
    let t_falsy = falsy
        .as_ref()
        .clone()
        .make_stable(input_domain.as_row_by_row(), input_metric.clone())?;

    let (truthy_domain, _truthy_metric) = t_truthy.output_space();
    let (falsy_domain, _falsy_metric) = t_falsy.output_space();

    if truthy_domain != falsy_domain {
        return fallible!(
            MakeTransformation,
            "output domains in ternary must match, instead found {:?} and {:?}",
            truthy_domain,
            falsy_domain
        );
    }

    if matches!(truthy_domain.column.dtype(), DataType::Categorical(_, _)) {
        return fallible!(MakeTransformation, "ternary cannot be applied to categorical data, because it may trigger a data-dependent CategoricalRemappingWarning in Polars");
    }

    let mut output_domain = truthy_domain.clone();
    // TODO: Cleanup output_domain?
    // output_domain.column.drop_bounds().ok();
    // output_domain.column.nullable = false;
    output_domain.context = input_domain.context.clone();

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(move |arg| {
            let predicate = t_predicate.invoke(arg)?;
            let truthy = t_truthy.invoke(arg)?;
            let falsy = t_falsy.invoke(arg)?;

            Ok(ExprPlan {
                plan: arg.clone(),
                expr: Expr::Ternary {
                    predicate: Arc::new(predicate.expr),
                    truthy: Arc::new(truthy.expr),
                    falsy: Arc::new(falsy.expr),
                },
                fill: None, // Ternary is run before aggregation, so there's no empty group that needs a default filled in.
            })
        }),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}
