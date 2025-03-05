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
    match &plan {
        // allow column selection, renaming in postprocessing
        // this postprocessor is always used by query plans translated from SQL queries
        #[cfg(feature = "contrib")]
        DslPlan::Select {
            expr: exprs, input, ..
        }
        | DslPlan::HStack { input, exprs, .. } => {
            if exprs
                .iter()
                .any(|e| !e.clone().meta().is_column_selection(true))
            {
                return Ok(None);
            }
            let m_in = input.as_ref().clone().make_private(
                input_domain,
                input_metric,
                output_measure,
                global_scale,
                threshold,
            )?;
            let post = match plan {
                DslPlan::Select { expr, options, .. } => {
                    Function::new_fallible(move |arg: &DslPlan| {
                        Ok(DslPlan::Select {
                            input: Arc::new(arg.clone()),
                            expr: expr.clone(),
                            options,
                        })
                    })
                }
                DslPlan::HStack { exprs, options, .. } => {
                    Function::new_fallible(move |arg: &DslPlan| {
                        Ok(DslPlan::HStack {
                            input: Arc::new(arg.clone()),
                            exprs: exprs.clone(),
                            options,
                        })
                    })
                }
                _ => unreachable!(),
            };
            m_in >> post
        }

        _ => return Ok(None),
    }
    .map(Some)
}
