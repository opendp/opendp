use std::collections::HashSet;

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{Bounds, FrameDistance};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use polars::prelude::*;

use super::StableDslPlan;

#[cfg(test)]
mod test;

/// Transformation for selecting columns in a microdata LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_select<MI: 'static + Metric, MO: UnboundedMetric>(
    input_domain: DslPlanDomain,
    input_metric: MI,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, MI, DslPlanDomain, FrameDistance<MO>>>
where
    DslPlan: StableDslPlan<MI, FrameDistance<MO>>,
    (DslPlanDomain, MI): MetricSpace,
    (DslPlanDomain, FrameDistance<MO>): MetricSpace,
{
    let DslPlan::Select {
        input,
        expr: exprs,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected select logical plan");
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

    let series_domains = t_exprs
        .iter()
        .map(|t| t.output_domain.column.clone())
        .collect();

    if let Some(identifier) = middle_metric.0.identifier() {
        let names = (t_exprs.iter())
            .map(|t| t.output_domain.column.name.clone())
            .collect::<HashSet<_>>();
        if !names.is_disjoint(&HashSet::from_iter(identifier.meta().root_names())) {
            return fallible!(
                MakeTransformation,
                "identifiers ({names:?}) may not be modified"
            );
        }
    }

    // A column is unchanged only when its original name and values are retained.
    fn is_identity(expr: &Expr, name: &PlSmallStr) -> bool {
        match expr {
            Expr::Alias(inner, _) => is_identity(inner, name),
            Expr::Column(column) => column == name,
            _ => false,
        }
    }

    let retained_columns = exprs
        .iter()
        .zip(t_exprs.iter())
        .filter(|(expr, t)| is_identity(expr, &t.output_domain.column.name))
        .map(|(_, t)| t.output_domain.column.name.clone())
        .collect::<HashSet<_>>();

    let changed_columns = middle_domain
        .series_domains
        .iter()
        .map(|s| s.name.clone())
        .chain(t_exprs.iter().map(|t| t.output_domain.column.name.clone()))
        .filter(|name| !retained_columns.contains(name))
        .collect::<HashSet<_>>();

    fn unaffected(by: &HashSet<Expr>, changed_columns: &HashSet<PlSmallStr>) -> bool {
        if changed_columns.is_empty() {
            return true;
        }
        // Selectors may depend on columns without exposing them through root_names().
        if by
            .iter()
            .any(|expr| expr.clone().meta().has_multiple_outputs())
        {
            return false;
        }
        let root_names = by
            .iter()
            .flat_map(|expr| expr.clone().meta().root_names())
            .collect::<HashSet<_>>();

        root_names.is_disjoint(changed_columns)
    }

    // A literal-only select may change the number of rows.
    let preserves_length = exprs
        .iter()
        .any(|expr| !expr.clone().meta().root_names().is_empty());

    let margins = middle_domain
        .margins
        .iter()
        .filter(|m| preserves_length && unaffected(&m.by, &changed_columns))
        .cloned()
        .collect();
    let output_domain = DslPlanDomain::new_with_margins(series_domains, margins)?;

    let t_select = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        middle_metric,
        Function::new_fallible(move |plan: &DslPlan| {
            let expr_arg = plan.clone();
            Ok(DslPlan::Select {
                input: Arc::new(plan.clone()),
                expr: (t_exprs.iter())
                    .map(|t| t.invoke(&expr_arg).map(|p| p.expr))
                    .collect::<Fallible<Vec<_>>>()?,
                options,
            })
        }),
        StabilityMap::new(move |bounds: &Bounds| {
            Bounds(
                bounds
                    .0
                    .iter()
                    .filter(|b| unaffected(&b.by, &changed_columns))
                    .cloned()
                    .collect(),
            )
        }),
    )?;

    t_prior >> t_select
}
