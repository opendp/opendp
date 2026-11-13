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
            checked_result(
                project_proof::construct(input.clone(), exprs.clone(), &source),
                &expected,
            )
        }),
        StabilityMap::new(global_bound),
    )?;
    prior >> step
}


/// Construct a certified concrete projection. Identity columns retain their
/// value bounds; derived expressions retain only dtype and nullability.
pub(super) fn make_stable_vtable_project(
    input_domain: crate::domains::verified_plan::VFrameDomain,
    metric: crate::metrics::SymmetricDistance,
    exprs: Vec<VExpr>,
) -> Fallible<Transformation<
    crate::domains::verified_plan::VFrameDomain, crate::metrics::SymmetricDistance,
    crate::domains::verified_plan::VFrameDomain, crate::metrics::SymmetricDistance,
>> {
    if !input_domain.margins.is_empty() {
        return fallible!(MakeTransformation,
            "verified table projection requires a margin-free domain");
    }
    let output_domain = project_domain(&input_domain, &exprs)?;
    let input_rules = crate::domains::verified_frame::verified_rules(
        &input_domain.series_domains)?;
    let output_rules = crate::domains::verified_frame::verified_rules(
        &output_domain.series_domains)?;
    let certified = project_proof::CertifiedTableProject::new(
        exprs, &input_rules, &output_rules,
    ).ok_or_else(|| err!(MakeTransformation,
        "projection output rules are not certified"))?;
    Transformation::new(
        input_domain, metric.clone(), output_domain, metric,
        Function::new_fallible(move |table: &verified::VTable| {
            certified.invoke(table).map_err(|_| {
                err!(MakeTransformation, "input outside certified projection domain")
            })
        }),
        StabilityMap::new(|d: &u32| table_proof::map_distance(*d)),
    )
}
