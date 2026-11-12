//! Executable constructors with Verus certificates for complete VPlans.
//! The kernel owns query semantics and typing; OpenDP owns stability proofs.

use super::verified::{self, VExpr, VField, VPlan};
use vstd::prelude::*;

verus! {

/// The shared contract of every successful make_* entry point.
pub open spec fn certified_result(
    result: Option<(VPlan, Vec<VField>)>, source_schema: Seq<VField>,
) -> bool {
    match result {
        Some((plan, fields)) =>
            verified::plan_schema(&plan, source_schema) == Some(fields@)
            && super::stability::stable_on_typed_insertions(&plan, source_schema),
        None => true,
    }
}

/// One type checker and one proof path for all constructors.
fn certify(plan: VPlan, source_schema: &Vec<VField>)
    -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_result(result, source_schema@),
{
    match verified::check_plan(&plan, source_schema) {
        None => None,
        Some(fields) => {
            proof {
                super::stability::lemma_all_typed_insertions_stable(&plan, source_schema@);
            }
            Some((plan, fields))
        },
    }
}

pub fn make_input(
    input: VPlan, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_result(result, source_schema@),
{
    certify(input, source_schema)
}

pub fn make_project(
    input: VPlan, exprs: Vec<VExpr>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_result(result, source_schema@),
{
    certify(VPlan::Project { input: Box::new(input), exprs }, source_schema)
}

pub fn make_filter(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures certified_result(result, source_schema@),
{
    certify(VPlan::Filter { input: Box::new(input), predicate }, source_schema)
}

} // verus!
