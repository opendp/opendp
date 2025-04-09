use std::collections::HashSet;
use std::sync::Arc;

use crate::combinators::{BasicCompositionMeasure, make_basic_composition};
use crate::core::{Function, Measurement, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::measurements::make_private_expr;
use crate::metrics::{Bounds, FrameDistance, L0PInfDistance, L01InfDistance};
use crate::transformations::StableDslPlan;
use crate::transformations::traits::UnboundedMetric;
use make_private_expr::PrivateExpr;
use polars::prelude::{DslPlan, Expr};

#[cfg(test)]
mod test;

/// Create a private version of an aggregate operation on a LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `output_measure` - The measure of the output LazyFrame.
/// * `plan` - The LazyFrame to transform.
/// * `global_scale` - The parameter for the measurement.
pub fn make_private_select<MI, MO>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<MI>,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
) -> Fallible<Measurement<DslPlanDomain, DslPlan, FrameDistance<MI>, MO>>
where
    MI: 'static + UnboundedMetric,
    MI::EventMetric: UnboundedMetric,
    MO: 'static + BasicCompositionMeasure,
    Expr: PrivateExpr<L01InfDistance<MI::EventMetric>, MO>,
    DslPlan: StableDslPlan<FrameDistance<MI>, FrameDistance<MI::EventMetric>>,
    (DslPlanDomain, FrameDistance<MI>): MetricSpace,
    (DslPlanDomain, FrameDistance<MI::EventMetric>): MetricSpace,
{
    let DslPlan::Select { expr, input, .. } = plan.clone() else {
        return fallible!(MakeMeasurement, "Expected selection in logical plan");
    };

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let margin = middle_domain.get_margin(&HashSet::new());

    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::Aggregation {
            margin: margin.clone(),
        },
    };

    // now that the domain is set up, we can clone it for use in the closure
    let margin = margin.clone();

    if margin.max_groups.unwrap_or(1) != 1 {
        return fallible!(
            MakeMeasurement,
            "There is only one partition in select, so both num_groups and max_groups must either be unset or one"
        );
    }

    let t_group_by = Transformation::new(
        middle_domain.clone(),
        expr_domain.clone(),
        Function::new(Clone::clone),
        middle_metric.clone(),
        L0PInfDistance(middle_metric.0.clone()),
        // the output distance triple consists of three numbers:
        // l0: number of changed partitions. Only one partition exists in select
        // l1: total number of contributions across all partitions
        // l∞: max number of contributions in any one partition
        StabilityMap::new_fallible(move |d_in: &Bounds| {
            let l1 = d_in
                .get_bound(&HashSet::new())
                .per_group
                .ok_or_else(|| err!(FailedMap, "per_group is unknown"))?;

            Ok((1, l1, l1))
        }),
    )?;

    let m_exprs = expr
        .into_iter()
        .map(|expr| {
            make_private_expr(
                expr_domain.clone(),
                L0PInfDistance(middle_metric.0.clone()),
                output_measure.clone(),
                expr.clone(),
                global_scale,
            )
        })
        .collect::<Fallible<_>>()?;
    let m_select_expr = (t_group_by >> make_basic_composition(m_exprs)?)?;

    let privacy_map = m_select_expr.privacy_map.clone();
    let m_select = Measurement::new(
        middle_domain,
        Function::new_fallible(move |arg: &DslPlan| {
            let mut output = plan.clone();
            if let DslPlan::Select {
                ref mut input,
                ref mut expr,
                ..
            } = output
            {
                *input = Arc::new(arg.clone());
                *expr = m_select_expr
                    .invoke(&arg)?
                    .into_iter()
                    .map(|e| e.expr)
                    .collect();
            };
            Ok(output)
        }),
        middle_metric,
        output_measure,
        privacy_map,
    )?;

    t_prior >> m_select
}
