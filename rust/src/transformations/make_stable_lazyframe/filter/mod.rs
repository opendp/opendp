use crate::core::{Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprContext, ExprDomain, LogicalPlanDomain};
use crate::error::*;
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::StableExpr;
use polars::prelude::*;

use super::StableLogicalPlan;

#[cfg(test)]
mod test;

/// Transformation for creating a stable LazyFrame filter.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_filter<M: Metric>(
    input_domain: LogicalPlanDomain,
    input_metric: M,
    plan: LogicalPlan,
) -> Fallible<Transformation<LogicalPlanDomain, LogicalPlanDomain, M, M>>
where
    M: UnboundedMetric + 'static,
    (LogicalPlanDomain, M): MetricSpace,
    LogicalPlan: StableLogicalPlan<M, M>,
    Expr: StableExpr<M, M>,
{
    let LogicalPlan::Filter { input, predicate } = plan else {
        return fallible!(MakeTransformation, "Expected Aggregate logical plan");
    };

    let t_prior = (input.as_ref().clone()).make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let expr_domain = ExprDomain::new(middle_domain.clone(), ExprContext::RowByRow);

    let t_pred = predicate
        .clone()
        .make_stable(expr_domain, middle_metric.clone())?;

    let pred_dtype = t_pred.output_domain.active_series()?.field.dtype.clone();

    if pred_dtype != DataType::Boolean {
        return fallible!(
            MakeTransformation,
            "Expected predicate to return a boolean value, got: {:?}",
            pred_dtype
        );
    }

    let mut output_domain = middle_domain.clone();

    output_domain.margins.values_mut().for_each(|m| {
        // After filtering you no longer know partition lengths or keys.
        m.public_info = None;
    });

    t_prior
        >> Transformation::new(
            middle_domain,
            output_domain,
            Function::new(move |plan: &LogicalPlan| LogicalPlan::Filter {
                input: Arc::new(plan.clone()),
                predicate: predicate.clone(),
            }),
            middle_metric.clone(),
            middle_metric,
            StabilityMap::new(Clone::clone),
        )?
}
