//! Executable projection node, checked in Verus before OpenDP wraps it.

use super::verified::{VExpr, VField, VPlan};
use super::stability;
use vstd::prelude::*;

verus! {

pub fn construct(
    input: VPlan, exprs: Vec<VExpr>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures stability::certified_result(result, source_schema@),
{
    let plan = VPlan::Project { input: Box::new(input), exprs };
    match stability::check_certified_plan(&plan, source_schema) {
        Some(fields) => Some((plan, fields)),
        None => None,
    }
}

} // verus!
