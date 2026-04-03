use crate::core::{Domain, Function, Metric, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::FrameDistance;
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use polars::prelude::*;

#[cfg(test)]
mod test;

pub(crate) fn make_chain_filter<DI, MI, MO>(
    t_prior: Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>,
    predicate: Expr,
) -> Fallible<Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>>
where
    DI: Domain + 'static,
    MI: Metric + 'static,
    MO: UnboundedMetric,
    (DI, MI): MetricSpace,
    (DslPlanDomain, FrameDistance<MO>): MetricSpace,
{
    let (middle_domain, middle_metric) = t_prior.output_space();

    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };

    let mut output_domain = middle_domain.clone();
    let t_pred = predicate.make_stable(expr_domain, middle_metric.clone())?;

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
        m.invariant = None;
    });

    make_chain_filter_unchecked(
        t_prior,
        middle_domain,
        middle_metric,
        output_domain,
        Function::new_fallible(move |plan: &DslPlan| {
            Ok(DslPlan::Filter {
                input: Arc::new(plan.clone()),
                predicate: function.eval(plan)?.expr,
            })
        }),
    )
}

pub(crate) fn make_chain_filter_unchecked<DI, MI, MO>(
    t_prior: Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>,
    middle_domain: DslPlanDomain,
    middle_metric: FrameDistance<MO>,
    output_domain: DslPlanDomain,
    function: Function<DslPlan, DslPlan>,
) -> Fallible<Transformation<DI, MI, DslPlanDomain, FrameDistance<MO>>>
where
    DI: Domain + 'static,
    MI: Metric + 'static,
    MO: UnboundedMetric,
    (DI, MI): MetricSpace,
    (DslPlanDomain, FrameDistance<MO>): MetricSpace,
{
    t_prior
        >> Transformation::new(
            middle_domain,
            middle_metric.clone(),
            output_domain,
            middle_metric,
            function,
            StabilityMap::new(Clone::clone),
        )?
}

/// Transformation for creating a stable LazyFrame filter.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_filter<MO>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<MO>,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, FrameDistance<MO>, DslPlanDomain, FrameDistance<MO>>>
where
    MO: UnboundedMetric,
{
    let DslPlan::Filter {
        input: _,
        predicate,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected filter in logical plan");
    };

    let t_prior = Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        input_domain,
        input_metric,
        Function::new(Clone::clone),
        StabilityMap::new(Clone::clone),
    )?;

    make_chain_filter(t_prior, predicate)
}
