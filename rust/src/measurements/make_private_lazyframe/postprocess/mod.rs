use std::sync::Arc;

use polars::prelude::DslPlan;
use polars_plan::prelude::ProjectionOptions;
use crate::polars::options::valid_projection_options;

use crate::combinators::CompositionMeasure;
use crate::core::{Metric, MetricSpace};
use crate::domains::{DslPlanDomain, LazyFrameDomain};
use crate::{
    core::{Function, Measurement},
    error::*,
};

use super::PrivateDslPlan;

#[cfg(test)]
mod test;

// This path permits only column selection and renaming. Parallelism changes
// scheduling; shape validation remains mandatory.
fn validated_projection_options(options: ProjectionOptions) -> Fallible<ProjectionOptions> {
    if !valid_projection_options(options) {
        return fallible!(MakeMeasurement, "Postprocessing requires duplicate checks and broadcasting");
    }
    Ok(options)
}

pub fn match_postprocess<MI: 'static + Metric, MO: 'static + CompositionMeasure>(
    input_domain: DslPlanDomain,
    input_metric: MI,
    output_measure: MO,
    plan: DslPlan,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Option<Measurement<DslPlanDomain, MI, MO, DslPlan>>>
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
                    let options = validated_projection_options(options)?;
                    Function::new_fallible(move |arg: &DslPlan| {
                        Ok(DslPlan::Select {
                            input: Arc::new(arg.clone()),
                            expr: expr.clone(),
                            options,
                        })
                    })
                }
                DslPlan::HStack { exprs, options, .. } => {
                    let options = validated_projection_options(options)?;
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
