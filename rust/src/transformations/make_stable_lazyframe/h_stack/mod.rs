use std::collections::{HashMap, HashSet};

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{Bounds, FrameDistance};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use polars::prelude::*;

use super::{StableDslPlan, is_identity, retain_bounds, retain_margins};

#[cfg(test)]
mod test;

/// Transformation for horizontal stacking of columns in a LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_h_stack<MI: 'static + Metric, MO: UnboundedMetric>(
    input_domain: DslPlanDomain,
    input_metric: MI,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, MI, DslPlanDomain, FrameDistance<MO>>>
where
    DslPlan: StableDslPlan<MI, FrameDistance<MO>>,
    Expr: StableExpr<FrameDistance<MO>, FrameDistance<MO>>,
    (DslPlanDomain, MI): MetricSpace,
    (DslPlanDomain, FrameDistance<MO>): MetricSpace,
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
        .iter()
        .cloned()
        .map(|expr| expr.make_stable(expr_domain.clone(), middle_metric.clone()))
        .collect::<Fallible<Vec<_>>>()?;

    if let Some(identifier) = middle_metric.0.identifier() {
        let names = (t_exprs.iter())
            .map(|t| t.output_domain.column.name.clone())
            .collect::<HashSet<_>>();
        let root_names = HashSet::from_iter(identifier.meta().root_names());
        if !names.is_disjoint(&root_names) {
            return fallible!(
                MakeTransformation,
                "identifiers ({root_names:?}) may not be modified"
            );
        }
    }

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

    let changed_columns = exprs
        .iter()
        .zip(t_exprs.iter())
        .filter(|(expr, t)| !is_identity(expr, &t.output_domain.column.name))
        .map(|(_, t)| t.output_domain.column.name.clone())
        .collect::<HashSet<_>>();

    // Column changes invalidate grouping information in both domains and distances.
    let margins = retain_margins(&middle_domain.margins, &changed_columns);

    // instead of using the public APIs that check invariants, directly populate the struct entries
    let output_domain = DslPlanDomain::new_with_margins(series_domains, margins)?;

    let t_with_columns = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        middle_metric,
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
        StabilityMap::new(move |bounds: &Bounds| retain_bounds(bounds, &changed_columns)),
    )?;

    t_prior >> t_with_columns
}
