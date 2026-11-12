//! Dataset-edit stability of row-local query plans, proved against kernel semantics.
//! This module lives in OpenDP; the query kernel contains only computation semantics.

use super::verified::*;
use vstd::multiset::Multiset;
use vstd::prelude::*;

verus! {

pub open spec fn dataset_spec(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
) -> Result<Multiset<Seq<Option<Value>>>, ()> {
    match relation_spec(plan, rows) {
        Ok(result) => Ok(result.to_multiset()),
        Err(_) => Err(()),
    }
}

pub open spec fn seq_one_insert(
    before: Seq<Seq<Option<Value>>>, after: Seq<Seq<Option<Value>>>,
) -> bool {
    before =~= after
        || exists|i: int, row: Seq<Option<Value>>|
            0 <= i <= before.len() && after =~= before.insert(i, row)
}

pub open spec fn bag_one_edit(
    left: Multiset<Seq<Option<Value>>>, right: Multiset<Seq<Option<Value>>>,
) -> bool {
    left =~= right
        || exists|row: Seq<Option<Value>>| right =~= left.insert(row)
        || exists|row: Seq<Option<Value>>| left =~= right.insert(row)
}

pub proof fn lemma_rows_insert_valid(
    rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>,
    index: int, row: Seq<Option<Value>>,
)
    requires
        rows_in_schema(rows, schema),
        row_in_schema(row, schema),
        0 <= index <= rows.len(),
    ensures rows_in_schema(rows.insert(index, row), schema),
{
    rows.insert_ensures(index, row);
    assert forall|i: int| 0 <= i < rows.insert(index, row).len()
        implies row_in_schema(rows.insert(index, row)[i], schema) by {
        if i == index {
            assert(rows.insert(index, row)[i] == row);
        } else if i < index {
            assert(rows.insert(index, row)[i] == rows[i]);
        } else {
            assert(rows.insert(index, row)[i] == rows[i - 1]);
        }
    }
}

pub proof fn lemma_rows_drop_last_valid(
    rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>,
)
    requires rows_in_schema(rows, schema), rows.len() > 0,
    ensures
        rows_in_schema(rows.drop_last(), schema),
        row_in_schema(rows.last(), schema),
{
    assert forall|i: int| 0 <= i < rows.drop_last().len()
        implies row_in_schema(rows.drop_last()[i], schema) by {
        assert(rows.drop_last()[i] == rows[i]);
    }
    assert(rows.last() == rows[rows.len() as int - 1]);
}

/// A prefix of the input cannot be influenced by later rows.
pub proof fn lemma_apply_prefix_ignores_last(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>, count: nat,
)
    requires count < rows.len(),
    ensures
        apply_prefix_spec(plan, rows, count)
            == apply_prefix_spec(plan, rows.drop_last(), count),
    decreases count,
{
    if count > 0 {
        lemma_apply_prefix_ignores_last(plan, rows, (count - 1) as nat);
        assert(rows.drop_last()[(count - 1) as int] == rows[(count - 1) as int]);
    }
}

/// Inserting before the final element commutes with appending that element.
pub proof fn lemma_seq_insert_push<T>(xs: Seq<T>, index: int, row: T, tail: T)
    requires 0 <= index <= xs.len(),
    ensures xs.insert(index, row).push(tail) =~= xs.push(tail).insert(index, row),
{
    xs.insert_ensures(index, row);
    xs.push(tail).insert_ensures(index, row);
    assert forall|i: int| 0 <= i < xs.insert(index, row).push(tail).len()
        implies xs.insert(index, row).push(tail)[i]
            == xs.push(tail).insert(index, row)[i] by {
        if i < index {
        } else if i == index {
        } else if i <= xs.len() {
        } else {
            assert(i == xs.len() + 1);
        }
    }
}

/// A single local operator preserves one-record insertion.
pub proof fn lemma_apply_insert(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>, schema: Seq<Field>,
    output_schema: Seq<Field>, index: int, row: Seq<Option<Value>>,
)
    requires
        rows_in_schema(rows, schema),
        row_in_schema(row, schema),
        local_schema(plan, schema) == Some(output_schema),
        0 <= index <= rows.len(),
    ensures
        match (
            apply_spec(plan, rows),
            apply_spec(plan, rows.insert(index, row)),
            local_row_spec(plan, row),
        ) {
            (Ok(before), Ok(after), Ok(Some(added))) =>
                exists|i: int| 0 <= i <= before.len() && after =~= before.insert(i, added),
            (Ok(before), Ok(after), Ok(None)) => after =~= before,
            _ => false,
        },
    decreases rows.len(),
{
    lemma_apply_prefix_type_sound(plan, rows, schema, output_schema, rows.len());
    lemma_rows_insert_valid(rows, schema, index, row);
    lemma_apply_prefix_type_sound(
        plan, rows.insert(index, row), schema, output_schema,
        rows.insert(index, row).len());
    lemma_local_type_sound(plan, schema, row, output_schema);
    if index == rows.len() {
        let appended = rows.push(row);
        assert(rows.insert(index, row) =~= appended);
        assert(appended.drop_last() =~= rows);
        lemma_apply_prefix_ignores_last(plan, appended, rows.len());
        assert(apply_prefix_spec(plan, appended, rows.len()) == apply_spec(plan, rows));
        match local_row_spec(plan, row) {
            Ok(Some(added)) => {
                let before = apply_spec(plan, rows).unwrap();
                assert(before.push(added) =~= before.insert(before.len() as int, added));
                assert(exists|i: int| 0 <= i <= before.len()
                    && before.push(added) =~= before.insert(i, added));
            },
            Ok(None) => {},
            Err(_) => { assert(false); },
        }
    } else {
        let tail = rows.last();
        let prefix = rows.drop_last();
        let inserted = rows.insert(index, row);
        assert(index <= prefix.len());
        assert(inserted.last() == tail);
        assert(inserted.drop_last() =~= prefix.insert(index, row));
        lemma_rows_drop_last_valid(rows, schema);
        lemma_rows_insert_valid(prefix, schema, index, row);
        lemma_apply_insert(plan, prefix, schema, output_schema, index, row);
        lemma_apply_prefix_ignores_last(plan, rows, prefix.len());
        lemma_apply_prefix_ignores_last(plan, inserted, prefix.insert(index, row).len());
        assert(apply_prefix_spec(plan, rows, prefix.len()) == apply_spec(plan, prefix));
        assert(apply_prefix_spec(plan, inserted, prefix.insert(index, row).len())
            == apply_spec(plan, prefix.insert(index, row)));
        let before_prefix = apply_spec(plan, prefix).unwrap();
        let after_prefix = apply_spec(plan, prefix.insert(index, row)).unwrap();
        lemma_local_type_sound(plan, schema, tail, output_schema);
        match (local_row_spec(plan, row), local_row_spec(plan, tail)) {
            (Ok(Some(added)), Ok(Some(tail_result))) => {
                let j = choose|j: int| 0 <= j <= before_prefix.len()
                    && after_prefix =~= before_prefix.insert(j, added);
                lemma_seq_insert_push(before_prefix, j, added, tail_result);
                assert(exists|i: int| 0 <= i <= before_prefix.push(tail_result).len()
                    && after_prefix.push(tail_result)
                        =~= before_prefix.push(tail_result).insert(i, added));
            },
            (Ok(Some(added)), Ok(None)) => {
                let j = choose|j: int| 0 <= j <= before_prefix.len()
                    && after_prefix =~= before_prefix.insert(j, added);
                assert(exists|i: int| 0 <= i <= before_prefix.len()
                    && after_prefix =~= before_prefix.insert(i, added));
            },
            (Ok(None), Ok(Some(_))) | (Ok(None), Ok(None)) => {
                assert(after_prefix =~= before_prefix);
            },
            _ => { assert(false); },
        }
    }
}

/// The relational composition of one-insertion-stable steps is stable.
pub proof fn lemma_relation_insert(
    plan: &Plan, rows: Seq<Seq<Option<Value>>>,
    schema: Seq<Field>, output_schema: Seq<Field>,
    index: int, row: Seq<Option<Value>>,
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
        Plan::Input => {
            assert(seq_one_insert(rows, rows.insert(index, row)));
        },
        Plan::Project { input, .. } | Plan::Filter { input, .. } => {
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
                let (j, inserted_row) = choose|j: int, inserted_row: Seq<Option<Value>>|
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
    before: Seq<Seq<Option<Value>>>, after: Seq<Seq<Option<Value>>>,
)
    requires seq_one_insert(before, after),
    ensures bag_one_edit(before.to_multiset(), after.to_multiset()),
{
    if !(before =~= after) {
        let (index, row) = choose|index: int, row: Seq<Option<Value>>|
            0 <= index <= before.len() && after =~= before.insert(index, row);
        lemma_seq_insert_bag(before, index, row);
    }
}

/// Typed row insertion is enough for the symmetric-distance bound.
pub open spec fn table_one_insert(before: &Table, after: &Table, index: int) -> bool {
    let earlier = rows_view(before.rows@);
    let later = rows_view(after.rows@);
    before.schema@ =~= after.schema@
        && 0 <= index <= earlier.len()
        && later.len() == earlier.len() + 1
        && later =~= earlier.insert(index, later[index])
}

pub proof fn lemma_typed_row_insert_stable(
    plan: &Plan, before: &Table, after: &Table, index: int,
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

} // verus!
