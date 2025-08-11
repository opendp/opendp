use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::FrameDistance;
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
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
pub fn make_stable_filter<MI: UnboundedMetric, MO: UnboundedMetric>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<MI>,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, FrameDistance<MI>, DslPlanDomain, FrameDistance<MO>>>
where
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MO>>,
{
    let DslPlan::Filter { input, predicate } = plan else {
        return fallible!(MakeTransformation, "Expected filter in logical plan");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };

    let mut output_domain = middle_domain.clone();
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
    let function = t_pred.function.clone();

    output_domain.margins.iter_mut().for_each(|m| {
        // After filtering you no longer know partition lengths or keys.
        m.invariant = None;
    });

    t_prior
        >> Transformation::new(
            middle_domain,
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::new_fallible(move |plan: &DslPlan| {
                Ok(DslPlan::Filter {
                    input: Arc::new(plan.clone()),
                    predicate: function.eval(plan)?.expr,
                })
            }),
            StabilityMap::new(Clone::clone),
        )?
}
