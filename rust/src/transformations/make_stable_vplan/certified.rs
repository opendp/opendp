//! Verified plan constructors. Input plans are checked once; extensions
//! are type-checked locally against the established child schema.
//! OpenDP supplies and enforces the richer source-relative domain contract.

use super::verified::{self, VExpr, VField, VPlan};
use vstd::prelude::*;

verus! {

pub open spec fn certified_plan(
    plan: &VPlan, fields: Seq<VField>, source_schema: Seq<VField>,
) -> bool {
    verified::plan_schema(plan, source_schema) == Some(fields)
        && super::stability::stable_on_typed_insertions(plan, source_schema)
}

pub open spec fn certified_result(
    result: Option<(VPlan, Vec<VField>)>, source_schema: Seq<VField>,
) -> bool {
    match result {
        Some((plan, fields)) => certified_plan(&plan, fields@, source_schema),
        None => true,
    }
}

pub open spec fn has_child_schema(
    plan: &VPlan, source_schema: Seq<VField>, child_schema: Seq<VField>,
) -> bool {
    match plan {
        VPlan::Input => false,
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } =>
            verified::plan_schema(input, source_schema) == Some(child_schema),
    }
}

/// The local certificate is unconditional. The complete-plan certificate
/// assumes the caller's plan satisfies the input domain's typed contract.
pub open spec fn certified_extension(
    result: Option<(VPlan, Vec<VField>)>,
    child_schema: Seq<VField>, source_schema: Seq<VField>,
) -> bool {
    match result {
        Some((plan, fields)) =>
            verified::local_schema(&plan, child_schema) == Some(fields@)
            && super::stability::stable_local_insertions(&plan, child_schema)
            && (has_child_schema(&plan, source_schema, child_schema)
                ==> certified_plan(&plan, fields@, source_schema)),
        None => true,
    }
}

/// Validate an arbitrary caller plan exactly once, at the identity leaf.
pub fn make_input(
    input: VPlan, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_result(result, source_schema@),
{
    match verified::check_plan(&input, source_schema) {
        None => None,
        Some(fields) => {
            proof {
                super::stability::lemma_all_typed_insertions_stable(&input, source_schema@);
            }
            Some((input, fields))
        },
    }
}

/// Compile a single additional operator without traversing its child.
fn certify_extension(
    plan: VPlan, child_schema: &Vec<VField>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_extension(result, child_schema@, source_schema@),
{
    let fields = match &plan {
        VPlan::Input => None,
        VPlan::Project { exprs, .. } =>
            verified::project::check_schema_prefix(exprs, child_schema, exprs.len()),
        VPlan::Filter { predicate, .. } =>
            verified::filter::check_schema(predicate, child_schema),
    };
    match fields {
        None => None,
        Some(fields) => {
            proof {
                assert(verified::local_schema(&plan, child_schema@) == Some(fields@));
                super::stability::lemma_all_local_insertions_stable(
                    &plan, child_schema@, fields@,
                );
                if has_child_schema(&plan, source_schema@, child_schema@) {
                    assert(verified::plan_schema(&plan, source_schema@) == Some(fields@));
                    super::stability::lemma_all_typed_insertions_stable(
                        &plan, source_schema@,
                    );
                }
            }
            Some((plan, fields))
        },
    }
}

pub fn make_project(
    input: VPlan, exprs: Vec<VExpr>,
    child_schema: &Vec<VField>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_extension(result, child_schema@, source_schema@),
{
    certify_extension(VPlan::Project { input: Box::new(input), exprs },
        child_schema, source_schema)
}

pub fn make_filter(
    input: VPlan, predicate: VExpr,
    child_schema: &Vec<VField>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_extension(result, child_schema@, source_schema@),
{
    certify_extension(VPlan::Filter { input: Box::new(input), predicate },
        child_schema, source_schema)
}

} // verus!
