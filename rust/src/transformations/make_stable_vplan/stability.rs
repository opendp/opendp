//! Symmetric-distance stability of typed plans, by composition of row-local maps.

use super::verified::*;
use vstd::prelude::*;

pub(crate) use super::verified::row_local as lifting;

#[path = "stability/row_local.rs"]
mod row_local;
pub use row_local::*;

verus! {

/// A constructed plan has its declared schema and contracts symmetric
/// distance for arbitrary typed input datasets, not only adjacent datasets.
pub open spec fn certified_result(
    result: Option<(VPlan, Vec<VField>)>, source_schema: Seq<VField>,
) -> bool {
    match result {
        Some((plan, fields)) =>
            plan_schema(&plan, source_schema) == Some(fields@)
            && stable_on_typed_rows(&plan, source_schema),
        None => true,
    }
}

/// Check a complete plan once, producing the shared schema and stability
/// certificate used by every node constructor.
pub fn check_certified_plan(
    plan: &VPlan, source_schema: &Vec<VField>,
) -> (result: Option<Vec<VField>>)
    ensures match result {
        Some(fields) =>
            plan_schema(plan, source_schema@) == Some(fields@)
                && stable_on_typed_rows(plan, source_schema@),
        None => true,
    },
{
    match check_plan(plan, source_schema) {
        None => None,
        Some(fields) => {
            proof { lemma_all_typed_rows_stable(plan, source_schema@); }
            Some(fields)
        },
    }
}

/// Every pair of typed input relations satisfies the identity stability map.
/// This includes both insertions and removals, duplicate rows, and all bounds.
pub open spec fn stable_on_typed_rows(
    plan: &VPlan, schema: Seq<VField>,
) -> bool {
    forall|left: Seq<Seq<Option<VValue>>>, right: Seq<Seq<Option<VValue>>>|
        rows_in_schema(left, schema) && rows_in_schema(right, schema)
        ==> match (relation_spec(plan, left), relation_spec(plan, right)) {
            (Ok(a), Ok(b)) =>
                symmetric_distance::distance(a.to_multiset(), b.to_multiset())
                    <= symmetric_distance::distance(
                        left.to_multiset(), right.to_multiset()),
            _ => false,
        }
}

/// Composition preserves nonexpansiveness. Each well-typed plan node is
/// a total deterministic map-or-drop operation on the child's output.
pub proof fn lemma_relation_contraction(
    plan: &VPlan,
    left: Seq<Seq<Option<VValue>>>,
    right: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>,
    output_schema: Seq<VField>,
)
    requires
        rows_in_schema(left, schema),
        rows_in_schema(right, schema),
        plan_schema(plan, schema) == Some(output_schema),
    ensures
        match (relation_spec(plan, left), relation_spec(plan, right)) {
            (Ok(a), Ok(b)) =>
                symmetric_distance::distance(a.to_multiset(), b.to_multiset())
                    <= symmetric_distance::distance(
                        left.to_multiset(), right.to_multiset()),
            _ => false,
        },
    decreases *plan,
{
    lemma_relation_type_sound(plan, left, schema, output_schema);
    lemma_relation_type_sound(plan, right, schema, output_schema);
    match plan {
        VPlan::Input => {},
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_relation_contraction(input, left, right, schema, child_schema);
            lemma_relation_type_sound(input, left, schema, child_schema);
            lemma_relation_type_sound(input, right, schema, child_schema);
            let child_left = relation_spec(input, left).unwrap();
            let child_right = relation_spec(input, right).unwrap();
            assert(local_schema(plan, child_schema) == Some(output_schema));
            lemma_apply_as_lift(plan, child_left, child_schema, output_schema);
            lemma_apply_as_lift(plan, child_right, child_schema, output_schema);
            let f = |row: Seq<Option<VValue>>| local_row_spec(plan, row).unwrap();
            symmetric_distance::lemma_lift_contraction(child_left, child_right, f);
            let out_left = relation_spec(plan, left).unwrap();
            let out_right = relation_spec(plan, right).unwrap();
            assert(out_left =~= lifting::lift(child_left, f));
            assert(out_right =~= lifting::lift(child_right, f));
            assert(symmetric_distance::distance(
                out_left.to_multiset(), out_right.to_multiset())
                <= symmetric_distance::distance(
                    child_left.to_multiset(), child_right.to_multiset()));
        },
    }
}

pub proof fn lemma_all_typed_rows_stable(
    plan: &VPlan, schema: Seq<VField>,
)
    requires plan_schema(plan, schema).is_Some(),
    ensures stable_on_typed_rows(plan, schema),
{
    let output_schema = plan_schema(plan, schema).unwrap();
    assert forall|left: Seq<Seq<Option<VValue>>>,
                  right: Seq<Seq<Option<VValue>>>|
        rows_in_schema(left, schema) && rows_in_schema(right, schema)
        implies match (relation_spec(plan, left), relation_spec(plan, right)) {
            (Ok(a), Ok(b)) =>
                symmetric_distance::distance(a.to_multiset(), b.to_multiset())
                    <= symmetric_distance::distance(
                        left.to_multiset(), right.to_multiset()),
            _ => false,
        } by {
        lemma_relation_contraction(plan, left, right, schema, output_schema);
    }
}

} // verus!
