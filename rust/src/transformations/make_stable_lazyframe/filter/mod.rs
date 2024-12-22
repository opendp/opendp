use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::StableExpr;
use polars::prelude::*;

use super::StableDslPlan;

#[cfg(test)]
mod test;

/// Transformation for creating a stable LazyFrame filter.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_filter<M: Metric>(
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
    let DslPlan::Filter { input, predicate } = plan else {
        return fallible!(MakeTransformation, "Expected filter in logical plan");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };

    let t_pred = predicate
        .clone()
        .make_stable(expr_domain, middle_metric.clone())?;

    let pred_dtype = t_pred.output_domain.column.dtype();

    if !pred_dtype.is_bool() {
        return fallible!(
            MakeTransformation,
            "Expected predicate to return a boolean value, got: {:?}",
            pred_dtype
        );
    }

    let mut output_domain = middle_domain.clone();

    output_domain.margins.iter_mut().for_each(|m| {
        // After filtering you no longer know partition lengths or keys.
        m.public_info = None;
    });

    t_prior
        >> Transformation::new(
            middle_domain,
            output_domain,
            Function::new_fallible(move |plan: &DslPlan| {
                let predicate = t_pred.invoke(plan)?.expr;
                Ok(DslPlan::Filter {
                    input: Arc::new(plan.clone()),
                    predicate: predicate.clone(),
                })
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
