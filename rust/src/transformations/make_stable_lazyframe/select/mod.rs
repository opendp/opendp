use std::collections::HashSet;

use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::FrameDistance;
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
        .into_iter()
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

    let output_domain = DslPlanDomain::new(series_domains)?;

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
        StabilityMap::new(Clone::clone),
    )?;

    t_prior >> t_select
}
