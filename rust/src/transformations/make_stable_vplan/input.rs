//! Source node: validate the caller's full OpenDP domain, then return it unchanged.

use super::*;
use crate::{
    core::{CertifiedTransformationParts, Domain, Function, StabilityMap, Transformation},
    domains::verified_frame::{kernel_schema, verified_rules},
};

/// An identity node whose function and map are checked in Verus using the
/// same row-domain rules as OpenDP's concrete frame membership.
struct CertifiedTableIdentity {
    rules: Vec<verified::domain::VColumnRule>,
}

impl CertifiedTransformationParts<
    VFrameDomain, SymmetricDistance, VFrameDomain, SymmetricDistance,
> for CertifiedTableIdentity {
    fn into_parts(
        self, input_domain: &VFrameDomain, output_domain: &VFrameDomain,
    ) -> Fallible<(
        Function<verified::VTable, verified::VTable>,
        StabilityMap<SymmetricDistance, SymmetricDistance>,
    )> {
        if !input_domain.margins.is_empty()
            || input_domain != output_domain
            || verified_rules(&input_domain.series_domains)? != self.rules
        {
            return fallible!(MakeTransformation,
                "certified identity does not match its frame domain");
        }
        Ok((
            Function::new_fallible(move |table: &verified::VTable| {
                table_proof::invoke_identity(&self.rules, table).map_err(|_| {
                    err!(MakeTransformation, "input outside certified identity domain")
                })
            }),
            StabilityMap::new(|d: &u32| table_proof::map_distance(*d)),
        ))
    }
}

pub(super) fn make_stable_vtable_input(
    domain: VFrameDomain,
    metric: SymmetricDistance,
) -> Fallible<Transformation<
    VFrameDomain, SymmetricDistance, VFrameDomain, SymmetricDistance,
>> {
    if !domain.margins.is_empty() {
        return fallible!(MakeTransformation,
            "verified table identity requires a margin-free domain");
    }
    let rules = verified_rules(&domain.series_domains)?;
    Transformation::new_certified(
        domain.clone(), metric.clone(), domain, metric,
        CertifiedTableIdentity { rules },
    )
}

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
            checked_result(input_proof::construct(input.clone(), &source), &expected)
        }),
        StabilityMap::new(Clone::clone),
    )
}
