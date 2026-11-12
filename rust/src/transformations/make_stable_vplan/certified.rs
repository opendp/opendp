//! Verus-verified constructor entry points used by OpenDP's make functions.
//!
//! Unlike a parallel model of the constructors, these functions actually
//! construct the VPlan values returned by the OpenDP transformations.
//! Dynamic FrameDomain values and OpenDP's Transformation wrapper remain
//! outside Verus and are checked separately by the Rust adapter.

use super::verified::{self, VExpr, VField, VPlan};
use vstd::prelude::*;

verus! {

pub fn make_input(
    input: VPlan, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        match result {
            Some((plan, fields)) =>
                verified::plan_schema(&plan, source_schema@) == Some(fields@)
                && super::stability::stable_on_typed_insertions(&plan, source_schema@)
                && super::stability::stable_local_from_source(&plan, source_schema@),
            None => true,
        },
{
    let result = verified::construct::make_input(input, source_schema);
    if let Some((plan, _)) = &result {
        proof {
            super::stability::lemma_all_typed_insertions_stable(plan, source_schema@);
            super::stability::lemma_well_typed_plan_locally_stable(plan, source_schema@);
        }
    }
    result
}

pub fn make_project(
    input: VPlan, exprs: Vec<VExpr>, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        match result {
            Some((plan, fields)) =>
                verified::plan_schema(&plan, source_schema@) == Some(fields@)
                && super::stability::stable_on_typed_insertions(&plan, source_schema@)
                && super::stability::stable_local_from_source(&plan, source_schema@),
            None => true,
        },
{
    let result = verified::construct::make_project(input, exprs, source_schema);
    if let Some((plan, _)) = &result {
        proof {
            super::stability::lemma_all_typed_insertions_stable(plan, source_schema@);
            super::stability::lemma_well_typed_plan_locally_stable(plan, source_schema@);
        }
    }
    result
}

pub fn make_filter(
    input: VPlan, predicate: VExpr, source_schema: &Vec<VField>,
) -> (result: Option<(VPlan, Vec<VField>)>)
    ensures
        match result {
            Some((plan, fields)) =>
                verified::plan_schema(&plan, source_schema@) == Some(fields@)
                && super::stability::stable_on_typed_insertions(&plan, source_schema@)
                && super::stability::stable_local_from_source(&plan, source_schema@),
            None => true,
        },
{
    let result = verified::construct::make_filter(input, predicate, source_schema);
    if let Some((plan, _)) = &result {
        proof {
            super::stability::lemma_all_typed_insertions_stable(plan, source_schema@);
            super::stability::lemma_well_typed_plan_locally_stable(plan, source_schema@);
        }
    }
    result
}

} // verus!
