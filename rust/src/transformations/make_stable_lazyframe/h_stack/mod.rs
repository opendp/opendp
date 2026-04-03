use std::collections::{HashMap, HashSet};

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

pub(crate) fn make_chain_h_stack<DI, MI, MO>(
    t_prior: Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>,
    exprs: Vec<Expr>,
    options: ProjectionOptions,
) -> Fallible<Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>>
where
    DI: Domain + 'static,
    MI: Metric + 'static,
    MO: UnboundedMetric + PolarsMetric,
    Expr: StableExpr<FrameDistance<MO>, FrameDistance<MO>>,
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

    let root_names = id_sites_root_names(&middle_metric.0.id_sites());
    if !root_names.is_empty() {
        let names = (t_exprs.iter())
            .map(|t| t.output_domain.column.name.clone())
            .collect::<HashSet<_>>();
        if !names.is_disjoint(&root_names) {
            return fallible!(
                MakeTransformation,
                "identifiers ({root_names:?}) may not be modified"
            );
        }
    }

    let mut series_domains = Vec::new();
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

    let new_series_names = new_series
        .map(|series_domain| col(series_domain.name.clone()))
        .collect();
    let margins = (middle_domain.margins.iter())
        .filter(|m| m.by.is_disjoint(&new_series_names))
        .cloned()
        .collect();

    let output_domain = DslPlanDomain::new_with_margins(series_domains, margins)?;

    t_prior
        >> Transformation::new(
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
                    options: options.clone(),
                })
            }),
            StabilityMap::new(Clone::clone),
        )?
}

/// Transformation for horizontal stacking of columns in a LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_h_stack<MO: UnboundedMetric + PolarsMetric>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<MO>,
    plan: DslPlan,
) -> Fallible<
    Transformation<DslPlanDomain, FrameDistance<MO>, DslPlanDomain, FrameDistance<MO>>,
>
where
    Expr: StableExpr<FrameDistance<MO>, FrameDistance<MO>>,
{
    let DslPlan::HStack {
        input: _,
        exprs,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected with_columns logical plan");
    };

    let t_prior = Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        input_domain,
        input_metric,
        Function::new(Clone::clone),
        StabilityMap::new(Clone::clone),
    )?;

    make_chain_h_stack(t_prior, exprs, options)
}
