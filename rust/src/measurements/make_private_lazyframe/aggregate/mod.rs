use std::collections::BTreeSet;

use crate::combinators::{make_basic_composition, BasicCompositionMeasure};
use crate::core::{Function, Measurement, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprContext, ExprDomain, LogicalPlanDomain};
use crate::error::*;
use crate::measurements::make_private_expr;
use crate::metrics::PartitionDistance;
use crate::traits::InfMul;
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{DatasetMetric, StableLogicalPlan};
use make_private_expr::PrivateExpr;
use polars::prelude::*;
use polars_plan::prelude::GroupbyOptions;

/// Create a private version of an aggregate operation on a LazyFrame.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `output_measure` - The measure of the output LazyFrame.
/// * `plan` - The LazyFrame to transform.
/// * `global_scale` - The parameter for the measurement.
pub fn make_private_aggregate<MS, MI, MO>(
    input_domain: LogicalPlanDomain,
    input_metric: MS,
    output_measure: MO,
    plan: LogicalPlan,
    global_scale: Option<f64>,
) -> Fallible<Measurement<LogicalPlanDomain, LogicalPlan, MS, MO>>
where
    MS: 'static + DatasetMetric,
    MI: 'static + UnboundedMetric,
    MO: 'static + BasicCompositionMeasure,
    Expr: PrivateExpr<PartitionDistance<MI>, MO>,
    LogicalPlan: StableLogicalPlan<MS, MI>,
    (LogicalPlanDomain, MS): MetricSpace,
    (LogicalPlanDomain, MI): MetricSpace,
    (ExprDomain, MI): MetricSpace,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
{
    let LogicalPlan::Aggregate {
        input,
        keys,
        aggs,
        apply,
        options,
        ..
    } = plan.clone()
    else {
        return fallible!(MakeMeasurement, "Expected Aggregate logical plan");
    };

    let t_prior = input.make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if options.as_ref() != &GroupbyOptions::default() {
        return fallible!(MakeMeasurement, "Unsupported options in logical plan. Do not optimize the lazyframe passed into the constructor. Options should be default, but are {:?}", options);
    }

    if apply.is_some() {
        return fallible!(MakeMeasurement, "Apply is not supported in logical plan");
    }

    let keys = keys
        .iter()
        .map(|e| {
            Ok(match e {
                Expr::Column(name) => vec![(*name).to_string()],
                Expr::Columns(names) => names.clone(),
                e => {
                    return fallible!(
                        MakeMeasurement,
                        "Expected column expression in keys, found {:?}",
                        e
                    )
                }
            })
        })
        .collect::<Fallible<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<BTreeSet<_>>();

    let margin = middle_domain
        .margins
        .get(&keys)
        .ok_or_else(|| err!(MakeMeasurement, "Failed to find margin for {:?}", keys))?
        .clone();

    let expr_domain = ExprDomain::new(
        middle_domain.clone(),
        ExprContext::Aggregate {
            grouping_columns: keys,
        },
    );

    let t_group_by = Transformation::new(
        expr_domain.clone(),
        expr_domain.clone(),
        Function::new(Clone::clone),
        middle_metric.clone(),
        PartitionDistance(middle_metric.clone()),
        StabilityMap::new_fallible(move |&d_in| {
            let l0 = margin.max_influenced_partitions.unwrap_or(d_in).min(d_in);
            let li = margin.max_partition_contributions.unwrap_or(d_in).min(d_in);
            let l1 = l0.inf_mul(&li)?.min(d_in);
            Ok((l0, l1, li))
        }),
    )?;

    let m_exprs = (t_group_by
        >> make_basic_composition(
            aggs.into_iter()
                .map(|expr| {
                    make_private_expr(
                        expr_domain.clone(),
                        PartitionDistance(middle_metric.clone()),
                        output_measure.clone(),
                        expr.clone(),
                        global_scale,
                    )
                })
                .collect::<Fallible<_>>()?,
        )?)?;

    let f_exprs = m_exprs.function.clone();
    let privacy_map = m_exprs.privacy_map.clone();

    t_prior
        >> Measurement::new(
            middle_domain,
            Function::new_fallible(move |arg: &LogicalPlan| {
                let mut output = plan.clone();
                if let LogicalPlan::Aggregate {
                    ref mut input,
                    ref mut aggs,
                    ..
                } = output
                {
                    *input = Box::new(arg.clone());
                    *aggs = f_exprs.eval(&(arg.clone(), all()))?;
                };
                Ok(output)
            }),
            middle_metric,
            output_measure,
            privacy_map,
        )?
}

#[cfg(test)]
mod test;
