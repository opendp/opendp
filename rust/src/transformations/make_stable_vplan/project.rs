//! Projection node: construct its child and compose a stable select step.

use super::*;
use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{verified_frame::kernel_schema, verified_plan::project_domain},
};

pub(super) fn make_project(
    input_domain: VPlanDomain,
    input_metric: VFrameMetric,
    child: VPlan,
    exprs: Vec<VExpr>,
) -> Fallible<VTransformation> {
    let prior = super::make_stable_vplan(input_domain, input_metric, child)?;
    let (domain, metric) = prior.output_space();
    let output = domain.with_output(project_domain(&domain.frame, &exprs)?);
    let source = kernel_schema(&domain.source.series_domains)?;
    let expected = kernel_schema(&output.frame.series_domains)?;
    let step = Transformation::new(
        domain, metric.clone(), output, metric,
        Function::new_fallible(move |input: &VPlan| {
            checked_plan(
                VPlan::Project {
                    input: Box::new(input.clone()), exprs: exprs.clone(),
                },
                &source, &expected,
            )
        }),
        StabilityMap::new(global_bound),
    )?;
    prior >> step
}
