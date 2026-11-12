//! Dataset-edit stability of row-local query plans, proved against kernel semantics.
//! This module lives in OpenDP; the query kernel contains only computation semantics.

use super::verified::*;
use vstd::multiset::Multiset;
use vstd::prelude::*;

#[path = "../../core/verus/row_local.rs"]
pub(crate) mod lifting;

#[path = "stability/row_local.rs"]
mod row_local;
pub use row_local::*;

verus! {

/// A successful executable node has the declared source-relative
/// schema and is 1-Lipschitz for both insertion and deletion adjacency.
pub open spec fn certified_result(
    result: Option<(VPlan, Vec<VField>)>, source_schema: Seq<VField>,
) -> bool {
    match result {
        Some((plan, fields)) =>
            plan_schema(&plan, source_schema) == Some(fields@)
            && stable_on_typed_edits(&plan, source_schema),
        None => true,
    }
}

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

pub open spec fn bag_one_insert(
    before: Multiset<Seq<Option<VValue>>>,
    after: Multiset<Seq<Option<VValue>>>,
) -> bool {
    exists|row: Seq<Option<VValue>>| after =~= before.insert(row)
}

pub open spec fn bag_one_edit(
    left: Multiset<Seq<Option<VValue>>>, right: Multiset<Seq<Option<VValue>>>,
) -> bool {
    left =~= right
        || bag_one_insert(left, right)
        || bag_one_insert(right, left)
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

/// Symmetric adjacency allows insertion or deletion of one typed row.
pub open spec fn table_one_edit(before: &VTable, after: &VTable, index: int) -> bool {
    table_one_insert(before, after, index) || table_one_insert(after, before, index)
}

pub proof fn lemma_bag_one_edit_symmetric(
    before: Multiset<Seq<Option<VValue>>>,
    after: Multiset<Seq<Option<VValue>>>,
)
    requires bag_one_edit(before, after),
    ensures bag_one_edit(after, before),
{
    hide(bag_one_insert);
    assert((before =~= after) <==> (after =~= before));
    assert(bag_one_edit(before, after) == bag_one_edit(after, before));
}

/// Insertion stability also proves deletion stability by swapping the tables.
pub proof fn lemma_typed_row_edit_stable(
    plan: &VPlan, before: &VTable, after: &VTable, index: int,
)
    requires
        table_valid(before),
        table_valid(after),
        table_one_edit(before, after, index),
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
    if table_one_insert(before, after, index) {
        lemma_typed_row_insert_stable(plan, before, after, index);
    } else {
        assert(table_one_insert(after, before, index));
        assert(after.schema@ == before.schema@);
        lemma_typed_row_insert_stable(plan, after, before, index);
        match (
            dataset_spec(plan, rows_view(before.rows@)),
            dataset_spec(plan, rows_view(after.rows@)),
        ) {
            (Ok(before_bag), Ok(after_bag)) => {
                lemma_bag_one_edit_symmetric(after_bag, before_bag);
            },
            _ => { assert(false); },
        }
    }
}

/// Verus certificate for both directions of unit symmetric-distance edits.
pub open spec fn stable_on_typed_edits(
    plan: &VPlan, schema: Seq<VField>,
) -> bool {
    forall|before: &VTable, after: &VTable, index: int|
        table_valid(before)
        && table_valid(after)
        && before.schema@ == schema
        && table_one_edit(before, after, index)
        ==> match (
            dataset_spec(plan, rows_view(before.rows@)),
            dataset_spec(plan, rows_view(after.rows@)),
        ) {
            (Ok(before_bag), Ok(after_bag)) =>
                bag_one_edit(before_bag, after_bag),
            _ => false,
        }
}

pub proof fn lemma_all_typed_edits_stable(
    plan: &VPlan, schema: Seq<VField>,
)
    requires plan_schema(plan, schema).is_Some(),
    ensures stable_on_typed_edits(plan, schema),
{
    assert forall|before: &VTable, after: &VTable, index: int|
        table_valid(before)
        && table_valid(after)
        && before.schema@ == schema
        && table_one_edit(before, after, index)
        implies match (
            dataset_spec(plan, rows_view(before.rows@)),
            dataset_spec(plan, rows_view(after.rows@)),
        ) {
            (Ok(before_bag), Ok(after_bag)) =>
                bag_one_edit(before_bag, after_bag),
            _ => false,
        } by {
        assert(plan_schema(plan, before.schema@).is_Some());
        lemma_typed_row_edit_stable(plan, before, after, index);
    }
}

} // verus!
