//! Source node: validate the caller's full OpenDP domain, then return it unchanged.

use super::*;
use crate::{
    core::{Domain, Function, StabilityMap, Transformation},
    domains::verified_frame::kernel_schema,
};

pub(super) fn make_input(
    domain: VPlanDomain,
    metric: VFrameMetric,
) -> Fallible<VTransformation> {
    let source = kernel_schema(&domain.source.series_domains)?;
    let expected = kernel_schema(&domain.frame.series_domains)?;
    let input_domain = domain.clone();
    Transformation::new(
        domain.clone(), metric.clone(), domain, metric,
        Function::new_fallible(move |input: &VPlan| {
            // Verus checks typed schemas; the OpenDP domain additionally
            // constrains names, atom bounds, and margins.
            if !input_domain.member(input)? {
                return fallible!(MakeTransformation,
                    "caller plan does not satisfy the verified input domain");
            }
            checked_plan(input.clone(), &source, &expected)
        }),
        StabilityMap::new(Clone::clone),
    )
}
