use std::sync::Arc;

use crate::combinators::BasicCompositionMeasure;
use crate::core::{Metric, MetricSpace};
use crate::domains::{DslPlanDomain, LazyFrameDomain};
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};

use polars_plan::plans::DslPlan;

use super::PrivateDslPlan;

#[cfg(test)]
mod test;

/// Since we're recursing through DSL trees that describe the computation plan,
/// and postprocessors are at the root of the tree,
/// we unfortunately need to build a whitelist of postprocessors.
///
/// This is a whitelist in the same code structure as in the case for expressions.
/// If a DSL branch is not considered postprocessing, then execution will continue in the parent function.
pub fn match_postprocess<MI: 'static + Metric, MO: 'static + BasicCompositionMeasure>(
    input_domain: DslPlanDomain,
    input_metric: MI,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Option<Measurement<DslPlanDomain, DslPlan, MI, MO>>>
where
    DslPlan: PrivateDslPlan<MI, MO>,
    (DslPlanDomain, MI): MetricSpace,
    (LazyFrameDomain, MI): MetricSpace,
{
    match plan {
        #[cfg(feature = "contrib")]
        DslPlan::Sort {
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
                threshold,
            )?;
            let sort = Function::new_fallible(move |arg: &DslPlan| {
                Ok(DslPlan::Sort {
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
