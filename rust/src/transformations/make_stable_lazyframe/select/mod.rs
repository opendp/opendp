use std::collections::HashSet;

use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{FrameDistance, PolarsMetric, id_sites_root_names};
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

    let protected_columns = id_sites_root_names(&middle_metric.0.id_sites());
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

/// Transformation for selecting columns in a microdata LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_select<MO: UnboundedMetric + PolarsMetric>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<MO>,
    plan: DslPlan,
) -> Fallible<
    Transformation<DslPlanDomain, FrameDistance<MO>, DslPlanDomain, FrameDistance<MO>>,
> {
    let DslPlan::Select {
        input: _,
        expr: exprs,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected select logical plan");
    };

    let t_prior = Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        input_domain,
        input_metric,
        Function::new(Clone::clone),
        StabilityMap::new(Clone::clone),
    )?;

    make_chain_select(t_prior, exprs, options)
}
