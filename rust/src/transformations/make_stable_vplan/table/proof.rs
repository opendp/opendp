//! Shared, concrete VTable transformation contract.
//!
//! Node proofs supply the source/output row-domain preservation lemma.
//! Metric-space validity and symmetric-distance contraction are proved once.

use super::verified::{self, VField, VPlan, VTable, VValue};
use super::verified::{domain, frame_metric_space, symmetric_distance, transformation};
use super::verified::domain::VColumnRule;
use super::stability;
use vstd::prelude::*;

verus! {

pub open spec fn valid_table_transformation(
    plan: &VPlan, input_rules: Seq<VColumnRule>, output_rules: Seq<VColumnRule>,
) -> bool {
    &&& frame_metric_space::symmetric_table_space(input_rules)
    &&& frame_metric_space::symmetric_rows_space(output_rules)
    &&& transformation::valid_transformation::<
        VTable, Seq<Seq<Option<VValue>>>, u32, u32, (), (),
    >(
        |table: VTable| domain::table_member(&table, input_rules),
        |rows: Seq<Seq<Option<VValue>>>| domain::rows_member(rows, output_rules),
        |left: VTable, right: VTable, d: u32|
            symmetric_distance::close(
                verified::rows_view(left.rows@).to_multiset(),
                verified::rows_view(right.rows@).to_multiset(), d),
        |left: Seq<Seq<Option<VValue>>>,
         right: Seq<Seq<Option<VValue>>>, d: u32|
            symmetric_distance::close(left.to_multiset(), right.to_multiset(), d),
        |_: u32| false,
        |table: VTable|
            verified::relation_spec(plan, verified::rows_view(table.rows@)),
        |d: u32| Ok(d),
    )
}


pub open spec fn certified_table_plan(
    plan: &VPlan, input_schema: Seq<VField>, output_schema: Seq<VField>,
    input_rules: Seq<VColumnRule>, output_rules: Seq<VColumnRule>,
) -> bool {
    &&& domain::schema_member(input_schema, input_rules)
    &&& domain::schema_member(output_schema, output_rules)
    &&& verified::plan_schema(plan, input_schema) == Some(output_schema)
    &&& forall|rows: Seq<Seq<Option<VValue>>>|
        domain::rows_member(rows, input_rules)
        ==> match verified::relation_spec(plan, rows) {
            Ok(output) => domain::rows_member(output, output_rules),
            Err(_) => false,
        }
}

/// One executable realization for any certified table-level plan. It
/// validates the source and returns the exact verified semantic relation.
pub fn invoke_table(
    plan: &VPlan, input_schema: &Vec<VField>, output_schema: &Vec<VField>,
    input_rules: &Vec<VColumnRule>, output_rules: &Vec<VColumnRule>,
    input: &VTable,
) -> (result: Result<VTable, ()>)
    ensures certified_table_plan(
        plan, input_schema@, output_schema@, input_rules@, output_rules@,
    ) ==> match result {
        Ok(output) =>
            domain::table_member(input, input_rules@)
            && domain::table_member(&output, output_rules@)
            && verified::relation_spec(plan, verified::rows_view(input.rows@))
                == Ok(verified::rows_view(output.rows@)),
        Err(_) => !domain::table_member(input, input_rules@),
    },
{
    if !domain::check_table(input, input_rules) {
        return Err(());
    }
    proof {
        domain::lemma_table_typed(input, input_rules@);
        if certified_table_plan(
            plan, input_schema@, output_schema@, input_rules@, output_rules@,
        ) {
            domain::lemma_unique_schema(
                input.schema@, input_schema@, input_rules@);
            verified::lemma_relation_type_sound(
                plan, verified::rows_view(input.rows@),
                input_schema@, output_schema@);
        }
    }
    let rows = match verified::evaluate(plan, input) {
        Ok(rows) => rows,
        Err(_) => {
            proof {
                if certified_table_plan(
                    plan, input_schema@, output_schema@, input_rules@, output_rules@,
                ) { assert(false); }
            }
            return Err(());
        },
    };
    let output = VTable { schema: output_schema.clone(), rows };
    proof {
        if certified_table_plan(
            plan, input_schema@, output_schema@, input_rules@, output_rules@,
        ) {
            domain::lemma_table_rows_domain(input, input_rules@);
            domain::lemma_table_rows_domain(&output, output_rules@);
        }
    }
    Ok(output)
}

pub fn map_distance(distance: u32) -> (output: u32)
    ensures output == distance,
{
    distance
}

/// Any domain-preserving, well-typed plan is a valid concrete transformation.
/// Operator-specific proofs need only establish row-domain preservation.
pub proof fn lemma_valid_table_transformation(
    plan: &VPlan,
    input_schema: Seq<VField>, output_schema: Seq<VField>,
    input_rules: Seq<VColumnRule>, output_rules: Seq<VColumnRule>,
)
    requires
        domain::schema_member(input_schema, input_rules),
        domain::schema_member(output_schema, output_rules),
        verified::plan_schema(plan, input_schema) == Some(output_schema),
        forall|rows: Seq<Seq<Option<VValue>>>|
            domain::rows_member(rows, input_rules)
            ==> match verified::relation_spec(plan, rows) {
                Ok(output) => domain::rows_member(output, output_rules),
                Err(_) => false,
            },
    ensures valid_table_transformation(plan, input_rules, output_rules),
{
    frame_metric_space::lemma_symmetric_table_space(input_rules);
    frame_metric_space::lemma_symmetric_rows_space(output_rules);
    assert forall|table: VTable|
        #[trigger] domain::table_member(&table, input_rules)
        implies match verified::relation_spec(
            plan, verified::rows_view(table.rows@)
        ) {
            Ok(output) => domain::rows_member(output, output_rules),
            Err(_) => true,
        } by {
        domain::lemma_table_rows_domain(&table, input_rules);
    }
    assert forall|left: VTable, right: VTable, d: u32|
        #[trigger] domain::table_member(&left, input_rules)
        && #[trigger] domain::table_member(&right, input_rules)
        && #[trigger] symmetric_distance::close(
            verified::rows_view(left.rows@).to_multiset(),
            verified::rows_view(right.rows@).to_multiset(), d)
        implies transformation::result_close(
            verified::relation_spec(plan, verified::rows_view(left.rows@)),
            verified::relation_spec(plan, verified::rows_view(right.rows@)), d,
            |a: Seq<Seq<Option<VValue>>>,
             b: Seq<Seq<Option<VValue>>>, bound: u32|
                symmetric_distance::close(a.to_multiset(), b.to_multiset(), bound),
            |_: u32| false,
        ) by {
        domain::lemma_table_rows_domain(&left, input_rules);
        domain::lemma_table_rows_domain(&right, input_rules);
        domain::lemma_unique_schema(left.schema@, input_schema, input_rules);
        domain::lemma_unique_schema(right.schema@, input_schema, input_rules);
        let a = verified::rows_view(left.rows@);
        let b = verified::rows_view(right.rows@);
        domain::lemma_rows_member_typed(a, input_schema, input_rules);
        domain::lemma_rows_member_typed(b, input_schema, input_rules);
        stability::lemma_relation_contraction(plan, a, b, input_schema, output_schema);
    }
}

} // verus!
