//! Executable identity node, checked in Verus before OpenDP wraps it.

use super::verified::{VField, VPlan};
use super::stability;
use vstd::prelude::*;

verus! {

pub fn construct(
    input: VPlan, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures stability::certified_result(result, source_schema@),
{
    match stability::check_certified_plan(&input, source_schema) {
        Some(fields) => Some((input, fields)),
        None => None,
    }
}

} // verus!
