use std::collections::HashSet;
use std::sync::Arc;

use crate::combinators::{make_basic_composition, BasicCompositionMeasure};
use crate::core::{Function, Measurement, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, WildExprDomain};
use crate::error::*;
use crate::measurements::make_private_expr;
use crate::metrics::PartitionDistance;
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{DatasetMetric, StableDslPlan};
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
pub fn make_private_select<MS, MI, MO>(
    input_domain: DslPlanDomain,
    input_metric: MS,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
) -> Fallible<Measurement<DslPlanDomain, DslPlan, MS, MO>>
where
    MS: 'static + DatasetMetric,
    MI: 'static + UnboundedMetric,
    MO: 'static + BasicCompositionMeasure,
    Expr: PrivateExpr<PartitionDistance<MI>, MO>,
    DslPlan: StableDslPlan<MS, MI>,
    (DslPlanDomain, MS): MetricSpace,
    (DslPlanDomain, MI): MetricSpace,
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

    if margin.max_partition_contributions.is_some() {
        return fallible!(
            MakeMeasurement,
            "Since there is only one partition in select, max_partition_contributions is redundant with the input distance, so it must be unset"
        );
    }

    if margin.max_influenced_partitions.unwrap_or(1) != 1
        || margin.max_num_partitions.unwrap_or(1) != 1
    {
        return fallible!(
            MakeMeasurement,
            "There is only one partition in select, so both max_influenced_partitions and max_num_partitions must either be unset or one"
        );
    }

    let t_group_by = Transformation::new(
        middle_domain.clone(),
        expr_domain.clone(),
        Function::new(Clone::clone),
        middle_metric.clone(),
        PartitionDistance(middle_metric.clone()),
        // the output distance triple consists of three numbers:
        // l0: number of changed partitions. Only one partition exists in select
        // l1: total number of contributions across all partitions
        // l∞: max number of contributions in any one partition
        StabilityMap::new(move |&d_in| (1, d_in, d_in)),
    )?;

    let m_exprs = expr
        .into_iter()
        .map(|expr| {
            make_private_expr(
                expr_domain.clone(),
                PartitionDistance(middle_metric.clone()),
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
