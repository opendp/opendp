//! Filter node: construct its child and compose a stable filter step.

use super::*;
use crate::{
    core::{Domain, Function, StabilityMap, Transformation},
    domains::{verified_frame::kernel_schema, verified_plan::filter_domain},
};

pub(super) fn make_filter(
    input_domain: VPlanDomain,
    input_metric: VFrameMetric,
    child: VPlan,
    predicate: VExpr,
) -> Fallible<VTransformation> {
    let prior = super::make_stable_vplan(input_domain, input_metric, child)?;
    let (domain, metric) = prior.output_space();
    let output = domain.with_output(filter_domain(&domain.frame, &predicate)?);
    let source = kernel_schema(&domain.source.series_domains)?;
    let grouping_schema = kernel_schema(&output.frame.series_domains)?;
    let output_contract = output.clone();
    let step = Transformation::new(
        domain, metric.clone(), output, metric,
        Function::new_fallible(move |input: &VPlan| {
            let Some((plan, _)) =
                filter_proof::construct(input.clone(), predicate.clone(), &source)
            else {
                return fallible!(MakeTransformation, "invalid verified filter plan");
            };
            // Enforce the complete source-relative output domain, rather than
            // merely checking that the inferred schema matches.
            if !output_contract.member(&plan)? {
                return fallible!(MakeTransformation,
                    "verified filter does not satisfy its output domain");
            }
            Ok(plan)
        }),
        // Each grouping key is evaluated row-locally by the verified kernel.
        // Reject untyped grouping expressions rather than applying the proof
        // outside its typing precondition.
        StabilityMap::new_fallible(move |bounds: &VFrameBounds| {
            for bound in &bounds.0 {
                let keys = bound.by.iter().cloned().collect::<Vec<_>>();
                if verified::project::check_schema_prefix(
                    &keys, &grouping_schema, keys.len(),
                ).is_none() {
                    return fallible!(FailedMap, "ill-typed verified grouping key");
                }
            }
            Ok(bounds.clone())
        }),
    )?;
    prior >> step
}
