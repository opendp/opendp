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

    // TODO: How to check if falsy is null, ie, "otherwise" is missing?
    // Something better than this:
    //
    // let t_falsy = (if falsy.to_string() != "null" {
    //     falsy
    //         .as_ref()
    //         .clone()
    //         .make_stable(input_domain.as_row_by_row(), input_metric.clone())?

    // TODO: If it is null, what is the transformation that should be used instead?
    // Or is a fix needed in make_stable?
    // (Using t_truthy can pass a test, but is probably wrong.)
    //
    // } else {
    //     t_truthy.clone()
    // });

    let (truthy_domain, truthy_metric) = t_truthy.output_space();
    let (falsy_domain, falsy_metric) = t_falsy.output_space();

    if truthy_domain != falsy_domain {
        return fallible!(
            MakeTransformation,
            "output domains in ternary must match, instead found {:?} and {:?}",
            truthy_domain,
            falsy_domain
        );
    }

    // TODO: How to exercise this? Is there a way to specify a user_distance should be used?
    if truthy_metric != falsy_metric {
        return fallible!(
            MakeTransformation,
            "output metrics in ternary must match, instead found {:?} and {:?}",
            truthy_metric,
            falsy_metric
        );
    }

    if matches!(truthy_domain.column.dtype(), DataType::Categorical(_, _)) {
        // Since literal categorical values aren't possible,
        // not clear if this is actually reachable.
        return fallible!(MakeTransformation, "ternary cannot be applied to categorical data, because it may trigger a data-dependent CategoricalRemappingWarning in Polars");
    }

    let mut output_domain = truthy_domain.clone();
    output_domain.column.drop_bounds().ok();
    output_domain.column.nullable = false;
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
