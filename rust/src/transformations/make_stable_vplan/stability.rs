//! Dataset-edit stability of row-local query plans, proved against kernel semantics.
//! This module lives in OpenDP; the query kernel contains only computation semantics.

use super::verified::*;
use vstd::multiset::Multiset;
use vstd::prelude::*;

#[path = "stability/row_local.rs"]
mod row_local;
use row_local::*;

verus! {

pub open spec fn dataset_spec(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
) -> Result<Multiset<Seq<Option<VValue>>>, ()> {
    match relation_spec(plan, rows) {
        Ok(result) => Ok(result.to_multiset()),
        Err(_) => Err(()),
    }
}

pub open spec fn seq_one_insert(
    before: Seq<Seq<Option<VValue>>>, after: Seq<Seq<Option<VValue>>>,
) -> bool {
    before =~= after
        || exists|i: int, row: Seq<Option<VValue>>|
            0 <= i <= before.len() && after =~= before.insert(i, row)
}

pub open spec fn bag_one_edit(
    left: Multiset<Seq<Option<VValue>>>, right: Multiset<Seq<Option<VValue>>>,
) -> bool {
    left =~= right
        || exists|row: Seq<Option<VValue>>| right =~= left.insert(row)
        || exists|row: Seq<Option<VValue>>| left =~= right.insert(row)
}

/// The relational composition of one-insertion-stable steps is stable.
pub proof fn lemma_relation_insert(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>, output_schema: Seq<VField>,
    index: int, row: Seq<Option<VValue>>,
)
    requires
        rows_in_schema(rows, schema),
        row_in_schema(row, schema),
        plan_schema(plan, schema) == Some(output_schema),
        0 <= index <= rows.len(),
    ensures
        match (
            relation_spec(plan, rows),
            relation_spec(plan, rows.insert(index, row)),
        ) {
            (Ok(before), Ok(after)) => seq_one_insert(before, after),
            _ => false,
        },
    decreases *plan,
{
    lemma_rows_insert_valid(rows, schema, index, row);
    lemma_relation_type_sound(plan, rows, schema, output_schema);
    lemma_relation_type_sound(
        plan, rows.insert(index, row), schema, output_schema);
    match plan {
        VPlan::Input => {
            assert(seq_one_insert(rows, rows.insert(index, row)));
        },
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } => {
            let child_schema = plan_schema(input, schema).unwrap();
            lemma_relation_insert(input, rows, schema, child_schema, index, row);
            lemma_relation_type_sound(input, rows, schema, child_schema);
            lemma_relation_type_sound(
                input, rows.insert(index, row), schema, child_schema);
            let before_child = relation_spec(input, rows).unwrap();
            let after_child = relation_spec(input, rows.insert(index, row)).unwrap();
            if before_child =~= after_child {
                assert(relation_spec(plan, rows) == relation_spec(plan, rows.insert(index, row)));
            } else {
                let (j, inserted_row) = choose|j: int, inserted_row: Seq<Option<VValue>>|
                    0 <= j <= before_child.len()
                        && after_child =~= before_child.insert(j, inserted_row);
                assert(rows_in_schema(after_child, child_schema));
                before_child.insert_ensures(j, inserted_row);
                assert(row_in_schema(inserted_row, child_schema));
                assert(rows_in_schema(before_child, child_schema));
                assert(local_schema(plan, child_schema) == Some(output_schema));
                lemma_apply_insert(
                    plan, before_child, child_schema, output_schema, j, inserted_row);
                match (
                    apply_spec(plan, before_child),
                    apply_spec(plan, after_child),
                    local_row_spec(plan, inserted_row),
                ) {
                    (Ok(before), Ok(after), Ok(Some(added))) => {
                        assert(seq_one_insert(before, after));
                    },
                    (Ok(before), Ok(after), Ok(None)) => {
                        assert(before =~= after);
                    },
                    _ => { assert(false); },
                }
            }
        },
    }
}

/// Moving an inserted row within a sequence does not alter its multiplicity.
pub proof fn lemma_seq_insert_bag<T>(xs: Seq<T>, index: int, value: T)
    requires 0 <= index <= xs.len(),
    ensures xs.insert(index, value).to_multiset() =~= xs.to_multiset().insert(value),
    decreases xs.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if index == xs.len() {
        assert(xs.insert(index, value) =~= xs.push(value));
        xs.to_multiset_ensures();
    } else {
        let tail = xs.last();
        let prefix = xs.drop_last();
        assert(index <= prefix.len());
        lemma_seq_insert_bag(prefix, index, value);
        assert(xs.insert(index, value).last() == tail);
        assert(xs.insert(index, value).drop_last() =~= prefix.insert(index, value));
        assert(xs.insert(index, value) =~= prefix.insert(index, value).push(tail));
        assert(xs =~= prefix.push(tail));
        prefix.to_multiset_ensures();
        prefix.insert(index, value).to_multiset_ensures();
        assert(xs.insert(index, value).to_multiset() =~=
            prefix.to_multiset().insert(value).insert(tail));
        assert(xs.to_multiset().insert(value) =~=
            prefix.to_multiset().insert(tail).insert(value));
        assert(prefix.to_multiset().insert(value).insert(tail) =~=
            prefix.to_multiset().insert(tail).insert(value));
    }
}

pub proof fn lemma_seq_one_insert_bag(
    before: Seq<Seq<Option<VValue>>>, after: Seq<Seq<Option<VValue>>>,
)
    requires seq_one_insert(before, after),
    ensures bag_one_edit(before.to_multiset(), after.to_multiset()),
{
    if !(before =~= after) {
        let (index, row) = choose|index: int, row: Seq<Option<VValue>>|
            0 <= index <= before.len() && after =~= before.insert(index, row);
        lemma_seq_insert_bag(before, index, row);
    }
}

/// Typed row insertion is enough for the symmetric-distance bound.
pub open spec fn table_one_insert(before: &VTable, after: &VTable, index: int) -> bool {
    let earlier = rows_view(before.rows@);
    let later = rows_view(after.rows@);
    before.schema@ =~= after.schema@
        && 0 <= index <= earlier.len()
        && later.len() == earlier.len() + 1
        && later =~= earlier.insert(index, later[index])
}

pub proof fn lemma_typed_row_insert_stable(
    plan: &VPlan, before: &VTable, after: &VTable, index: int,
)
    requires
        table_valid(before),
        table_valid(after),
        table_one_insert(before, after, index),
        plan_schema(plan, before.schema@).is_Some(),
    ensures
        match (
            dataset_spec(plan, rows_view(before.rows@)),
            dataset_spec(plan, rows_view(after.rows@)),
        ) {
            (Ok(before_bag), Ok(after_bag)) =>
                bag_one_edit(before_bag, after_bag),
            _ => false,
        },
{
    let previous = rows_view(before.rows@);
    let next = rows_view(after.rows@);
    let output_schema = plan_schema(plan, before.schema@).unwrap();
    assert(plan_schema(plan, after.schema@) == Some(output_schema));
    assert(next =~= previous.insert(index, next[index]));
    assert(row_in_schema(next[index], before.schema@));
    lemma_relation_insert(
        plan, previous, before.schema@, output_schema, index, next[index]);
    match (
        relation_spec(plan, previous),
        relation_spec(plan, next),
    ) {
        (Ok(before_rows), Ok(after_rows)) => {
            lemma_seq_one_insert_bag(before_rows, after_rows);
        },
        _ => { assert(false); },
    }
}

/// A compiled, well-typed plan is 1-Lipschitz for insert/delete
/// adjacency on the typed source tables.
///
/// This is quantified over *all* valid table pairs, so a constructor can
/// return this result as a postcondition rather than verifying one example.
pub open spec fn stable_on_typed_insertions(
    plan: &VPlan, schema: Seq<VField>,
) -> bool {
    forall|before: &VTable, after: &VTable, index: int|
        table_valid(before)
        && table_valid(after)
        && before.schema@ == schema
        && table_one_insert(before, after, index)
        ==> match (
            dataset_spec(plan, rows_view(before.rows@)),
            dataset_spec(plan, rows_view(after.rows@)),
        ) {
            (Ok(before_bag), Ok(after_bag)) =>
                bag_one_edit(before_bag, after_bag),
            _ => false,
        }
}

/// This proof is consumed by the executable, verified make_* entry points.
pub proof fn lemma_all_typed_insertions_stable(
    plan: &VPlan, schema: Seq<VField>,
)
    requires plan_schema(plan, schema).is_Some(),
    ensures stable_on_typed_insertions(plan, schema),
{
    assert forall|before: &VTable, after: &VTable, index: int|
        table_valid(before)
        && table_valid(after)
        && before.schema@ == schema
        && table_one_insert(before, after, index)
        implies match (
            dataset_spec(plan, rows_view(before.rows@)),
            dataset_spec(plan, rows_view(after.rows@)),
        ) {
            (Ok(before_bag), Ok(after_bag)) =>
                bag_one_edit(before_bag, after_bag),
            _ => false,
        } by {
        assert(plan_schema(plan, before.schema@).is_Some());
        lemma_typed_row_insert_stable(plan, before, after, index);
    }
}

/// One relational operator, applied to an arbitrary child relation, is
/// 1-Lipschitz under a single row insertion. This is the property required
/// by make_project and make_filter, independent of the child's AST.
pub open spec fn stable_local_insertions(
    plan: &VPlan, input_schema: Seq<VField>, output_schema: Seq<VField>,
) -> bool {
    forall|rows: Seq<Seq<Option<VValue>>>, index: int, row: Seq<Option<VValue>>|
        rows_in_schema(rows, input_schema)
        && row_in_schema(row, input_schema)
        && 0 <= index <= rows.len()
        ==> match (apply_spec(plan, rows), #[trigger] apply_spec(plan, rows.insert(index, row))) {
            (Ok(before), Ok(after)) => seq_one_insert(before, after),
            _ => false,
        }
}

pub proof fn lemma_all_local_insertions_stable(
    plan: &VPlan, input_schema: Seq<VField>, output_schema: Seq<VField>,
)
    requires local_schema(plan, input_schema) == Some(output_schema),
    ensures stable_local_insertions(plan, input_schema, output_schema),
{
    assert forall|rows: Seq<Seq<Option<VValue>>>, index: int, row: Seq<Option<VValue>>|
        rows_in_schema(rows, input_schema)
        && row_in_schema(row, input_schema)
        && 0 <= index <= rows.len()
        implies match (apply_spec(plan, rows), #[trigger] apply_spec(plan, rows.insert(index, row))) {
            (Ok(before), Ok(after)) => seq_one_insert(before, after),
            _ => false,
        } by {
        lemma_apply_insert(plan, rows, input_schema, output_schema, index, row);
        match (
            apply_spec(plan, rows),
            apply_spec(plan, rows.insert(index, row)),
            local_row_spec(plan, row),
        ) {
            (Ok(before), Ok(after), Ok(Some(added))) => {
                let j = choose|j: int|
                    0 <= j <= before.len() && after =~= before.insert(j, added);
                assert(seq_one_insert(before, after));
            },
            (Ok(before), Ok(after), Ok(None)) => {
                assert(before =~= after);
                assert(seq_one_insert(before, after));
            },
            _ => { assert(false); },
        }
    }
}

/// The local operator represented by any well-typed plan is stable. The
/// constructor certificates ensure this for the actual plan they return.
pub open spec fn stable_local_from_source(
    plan: &VPlan, source_schema: Seq<VField>,
) -> bool {
    match plan {
        VPlan::Input => stable_local_insertions(plan, source_schema, source_schema),
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } =>
            match (plan_schema(input, source_schema), plan_schema(plan, source_schema)) {
                (Some(child), Some(output)) => stable_local_insertions(plan, child, output),
                _ => false,
            },
    }
}

pub proof fn lemma_well_typed_plan_locally_stable(
    plan: &VPlan, source_schema: Seq<VField>,
)
    requires plan_schema(plan, source_schema).is_Some(),
    ensures stable_local_from_source(plan, source_schema),
{
    match plan {
        VPlan::Input => {
            lemma_all_local_insertions_stable(plan, source_schema, source_schema);
        },
        VPlan::Project { input, .. } | VPlan::Filter { input, .. } => {
            let child_schema = plan_schema(input, source_schema).unwrap();
            let output_schema = plan_schema(plan, source_schema).unwrap();
            assert(local_schema(plan, child_schema) == Some(output_schema));
            lemma_all_local_insertions_stable(plan, child_schema, output_schema);
        },
    }
}

} // verus!
