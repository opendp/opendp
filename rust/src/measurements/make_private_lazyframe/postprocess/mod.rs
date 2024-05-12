use std::sync::Arc;

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
            slice,
            sort_options,
        } => {
            let m_in = input.as_ref().clone().make_private(
                input_domain,
                input_metric,
                output_measure,
                global_scale,
            )?;
            let sort = Function::new_fallible(move |arg: &LogicalPlan| {
                Ok(LogicalPlan::Sort {
                    input: Arc::new(arg.clone()),
                    by_column: by_column.clone(),
                    slice: slice.clone(),
                    sort_options: sort_options.clone(),
                })
            });
            m_in >> sort
        }

        _ => return Ok(None),
    }
    .map(Some)
}
