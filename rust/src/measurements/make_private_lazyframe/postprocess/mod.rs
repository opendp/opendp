use crate::combinators::BasicCompositionMeasure;
use crate::core::{Metric, MetricSpace};
use crate::domains::{LazyFrameDomain, LogicalPlanDomain};
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};

use polars_plan::logical_plan::LogicalPlan;

use super::PrivateLogicalPlan;

#[cfg(test)]
mod test;

/// Since we're recursing through DSL trees that describe the computation plan,
/// and postprocessors are at the root of the tree,
/// we unfortunately need to build a whitelist of postprocessors.
///
/// This is a whitelist in the same code structure as in the case for expressions.
/// If a DSL branch is not considered postprocessing, then execution will continue in the parent function.
pub fn match_postprocess<MI: 'static + Metric, MO: 'static + BasicCompositionMeasure>(
    input_domain: LogicalPlanDomain,
    input_metric: MI,
    output_measure: MO,
    plan: LogicalPlan,
    global_scale: Option<f64>,
) -> Fallible<Option<Measurement<LogicalPlanDomain, LogicalPlan, MI, MO>>>
where
    LogicalPlan: PrivateLogicalPlan<MI, MO>,
    (LogicalPlanDomain, MI): MetricSpace,
    (LazyFrameDomain, MI): MetricSpace,
{
    match plan {
        #[cfg(feature = "contrib")]
        LogicalPlan::Sort {
            input,
            by_column,
            args,
        } => {
            let m_in =
                input.make_private(input_domain, input_metric, output_measure, global_scale)?;
            let sort = Function::new_fallible(move |arg: &LogicalPlan| {
                Ok(LogicalPlan::Sort {
                    input: Box::new(arg.clone()),
                    by_column: by_column.clone(),
                    args: args.clone(),
                })
            });
            m_in >> sort
        }

        _ => return Ok(None),
    }
    .map(Some)
}
