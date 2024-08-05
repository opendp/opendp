use std::collections::BTreeSet;
use std::mem::replace;

use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric};
use crate::error::*;
use crate::polars::ExprFunction;
use crate::transformations::DatasetMetric;

use super::StableExpr;

#[cfg(test)]
mod test;

/// Make a Transformation that renames a column in a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The alias expression
pub fn make_expr_alias<M: OuterMetric>(
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
    let Expr::Alias(input, name) = expr else {
        return fallible!(MakeTransformation, "expected alias expression");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let old_name = replace(
        &mut output_domain.active_series_mut()?.field.name,
        name.as_ref().into(),
    );

    // only keep margins with as many unique grouping keys as there were before
    // if the number of unique grouping keys drops after aliasing,
    //    then one of the grouping keys is shadowing another grouping key
    output_domain.frame_domain.margins = (output_domain.frame_domain.margins)
        .into_iter()
        .filter_map(|(k, v)| {
            let old_len = k.len();
            let new_k: BTreeSet<_> = (k.into_iter())
                .map(|col| {
                    if col == old_name {
                        name.to_string()
                    } else {
                        col
                    }
                })
                .collect();
            (new_k.len() == old_len).then_some((new_k, v))
        })
        .collect();

    let t_alias = Transformation::new(
        middle_domain.clone(),
        output_domain,
        Function::then_expr(move |expr| expr.alias(name.as_ref())),
        middle_metric.clone(),
        middle_metric,
        StabilityMap::new(Clone::clone),
    )?;

    t_prior >> t_alias
}
