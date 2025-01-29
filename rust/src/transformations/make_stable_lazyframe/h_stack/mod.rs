use std::collections::HashMap;

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::StableExpr;
use polars::prelude::*;

use super::StableDslPlan;

#[cfg(test)]
mod test;

/// Transformation for horizontal stacking of columns in a LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_h_stack<M: Metric>(
    input_domain: DslPlanDomain,
    input_metric: M,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, DslPlanDomain, M, M>>
where
    M: UnboundedMetric + 'static,
    (DslPlanDomain, M): MetricSpace,
    DslPlan: StableDslPlan<M, M>,
    Expr: StableExpr<M, M>,
{
    let DslPlan::HStack {
        input,
        exprs,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected with_columns logical plan");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    // create a transformation for each expression
    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };
    let t_exprs = exprs
        .into_iter()
        .map(|expr| expr.make_stable(expr_domain.clone(), middle_metric.clone()))
        .collect::<Fallible<Vec<_>>>()?;

    // expand and update the set of series domains on the output domain
    let mut series_domains = Vec::new();
    // keys are the column name, values are the index of the column
    let mut lookup = HashMap::new();

    let new_series = t_exprs.iter().map(|t| &t.output_domain.column);

    (middle_domain.series_domains.iter())
        .chain(new_series.clone())
        .for_each(|series_domain| {
            lookup
                .entry(series_domain.name.to_string())
                .and_modify(|i| {
                    series_domains[*i] = series_domain.clone();
                })
                .or_insert_with(|| {
                    series_domains.push(series_domain.clone());
                    series_domains.len() - 1
                });
        });

    // only keep margins for series that have not changed
    let new_series_names = new_series
        .map(|series_domain| col(series_domain.name.clone()))
        .collect();
    let margins = (middle_domain.margins.iter())
        .filter(|m| m.by.is_disjoint(&new_series_names))
        .cloned()
        .collect();

    // instead of using the public APIs that check invariants, directly populate the struct entries
    let output_domain = DslPlanDomain::new_with_margins(series_domains, margins)?;

    let t_with_columns = Transformation::new(
        middle_domain,
        output_domain,
        Function::new_fallible(move |plan: &DslPlan| {
            let expr_arg = plan.clone();
            Ok(DslPlan::HStack {
                input: Arc::new(plan.clone()),
                exprs: (t_exprs.iter())
                    .map(|t| t.invoke(&expr_arg).map(|p| p.expr))
                    .collect::<Fallible<Vec<_>>>()?,
                options,
            })
        }),
        middle_metric.clone(),
        middle_metric,
        StabilityMap::new(Clone::clone),
    )?;

    t_prior >> t_with_columns
}
