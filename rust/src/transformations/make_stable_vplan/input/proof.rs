//! Executable identity node, checked in Verus before OpenDP wraps it.

use super::verified::{self, VField, VPlan};
use super::stability;
use vstd::prelude::*;

verus! {

pub fn construct(
    input: VPlan, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures stability::certified_result(result, source_schema@),
{
    match verified::check_plan(&input, source_schema) {
        None => None,
        Some(fields) => {
            proof {
                stability::lemma_all_typed_edits_stable(&input, source_schema@);
            }
            Some((input, fields))
        },
    }
}

} // verus!
