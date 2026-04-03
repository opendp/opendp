use std::sync::Arc;

use polars::prelude::DslPlan;

use crate::combinators::CompositionMeasure;
use crate::core::{Metric, MetricSpace};
use crate::domains::{DslPlanDomain, LazyFrameDomain};
use crate::{
    core::{Domain, Function, Measurement},
    error::Fallible,
};

use super::PrivateDslPlan;

#[cfg(test)]
mod test;

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
    match_postprocess_with(
        |plan| {
            plan.make_private(
                input_domain.clone(),
                input_metric.clone(),
                output_measure.clone(),
                global_scale,
                threshold,
            )
        },
        plan,
    )
}

pub fn match_postprocess_with<DI, MI, MO, F>(
    recurse: F,
    plan: DslPlan,
) -> Fallible<Option<Measurement<DI, MI, MO, DslPlan>>>
where
    DI: Domain + 'static,
    MI: 'static + Metric,
    MO: 'static + CompositionMeasure,
    (DI, MI): MetricSpace,
    F: FnOnce(DslPlan) -> Fallible<Measurement<DI, MI, MO, DslPlan>>,
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
            let m_in = recurse(input.as_ref().clone())?;
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
