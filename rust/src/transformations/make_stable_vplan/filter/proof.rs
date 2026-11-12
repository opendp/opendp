//! Executable filter node, checked in Verus before OpenDP wraps it.

use super::verified::{self, VExpr, VField, VPlan};
use super::stability;
use vstd::prelude::*;

verus! {

pub fn construct(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures stability::certified_result(result, source_schema@),
{
    let plan = VPlan::Filter { input: Box::new(input), predicate };
    match verified::check_plan(&plan, source_schema) {
        None => None,
        Some(fields) => {
            proof {
                stability::lemma_all_typed_edits_stable(&plan, source_schema@);
            }
            Some((plan, fields))
        },
    }
}

} // verus!
