use std::collections::HashSet;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{bindings_root_names, claims_root_names, FrameDistance, PolarsMetric};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use polars::prelude::*;
use polars_plan::prelude::ProjectionOptions;

#[cfg(test)]
mod test;

pub(crate) fn make_chain_select<DI, MI, MO>(
    t_prior: Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>,
    exprs: Vec<Expr>,
    options: ProjectionOptions,
) -> Fallible<Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>>
where
    DI: Domain + 'static,
    MI: Metric + 'static,
    MO: UnboundedMetric + PolarsMetric,
    (DI, MI): MetricSpace,
    (DslPlanDomain, FrameDistance<MO>): MetricSpace,
{
    let (middle_domain, middle_metric) = t_prior.output_space();

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

    let protected_columns = bindings_root_names(&middle_metric.0.bindings())
        .into_iter()
        .chain(claims_root_names(&middle_metric.0.owner_claims()))
        .collect::<HashSet<_>>();
    if !protected_columns.is_empty() {
        let names = (t_exprs.iter())
            .map(|t| t.output_domain.column.name.clone())
            .collect::<HashSet<_>>();
        if !names.is_disjoint(&protected_columns) {
            return fallible!(
                MakeTransformation,
                "identifiers ({protected_columns:?}) may not be modified"
            );
        }
    }

    let output_domain = DslPlanDomain::new(series_domains)?;

    t_prior
        >> Transformation::new(
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
                    options: options.clone(),
                })
            }),
            StabilityMap::new(Clone::clone),
        )?
}
