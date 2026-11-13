//! Stability of a row-local step on *implied collected data*.
//!
//! The two input plans may differ, and their source bindings may denote
//! different datasets. Only their realized input relations need to share the
//! same domain. This is the lifting missing from the previous VPlan metric.

use super::verified::{self, VField, VPlan, VValue};
use super::stability;
use vstd::prelude::*;

verus! {

pub proof fn lemma_bound_step_contraction(
    left_plan: &VPlan, right_plan: &VPlan, step: &VPlan,
    left_source: Seq<Seq<Option<VValue>>>,
    right_source: Seq<Seq<Option<VValue>>>,
    left_source_schema: Seq<VField>,
    right_source_schema: Seq<VField>,
    input_schema: Seq<VField>,
    output_schema: Seq<VField>,
)
    requires
        verified::rows_in_schema(left_source, left_source_schema),
        verified::rows_in_schema(right_source, right_source_schema),
        verified::plan_schema(left_plan, left_source_schema) == Some(input_schema),
        verified::plan_schema(right_plan, right_source_schema) == Some(input_schema),
        verified::plan_schema(step, input_schema) == Some(output_schema),
    ensures
        match (
            verified::relation_spec(left_plan, left_source),
            verified::relation_spec(right_plan, right_source),
        ) {
            (Ok(left), Ok(right)) =>
                match (
                    verified::relation_spec(step, left),
                    verified::relation_spec(step, right),
                ) {
                    (Ok(out_left), Ok(out_right)) =>
                        verified::symmetric_distance::distance(
                            out_left.to_multiset(), out_right.to_multiset(),
                        ) <= verified::symmetric_distance::distance(
                            left.to_multiset(), right.to_multiset(),
                        ),
                    _ => false,
                },
            _ => false,
        },
{
    verified::lemma_relation_type_sound(
        left_plan, left_source, left_source_schema, input_schema);
    verified::lemma_relation_type_sound(
        right_plan, right_source, right_source_schema, input_schema);
    let left = verified::relation_spec(left_plan, left_source).unwrap();
    let right = verified::relation_spec(right_plan, right_source).unwrap();
    stability::lemma_relation_contraction(
        step, left, right, input_schema, output_schema,
    );
}

} // verus!
