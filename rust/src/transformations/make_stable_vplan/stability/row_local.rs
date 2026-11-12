//! A generic one-row-local operator changes at most one output row
//! when an input row is inserted. No operator variants are matched here.

use super::*;

verus! {

pub proof fn lemma_rows_insert_valid(
    rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    index: int, row: Seq<Option<VValue>>,
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
    rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
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
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>, count: nat,
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

/// A single local operator preserves one-record insertion.
/// The verified row evaluator agrees with the generic row-local lifting
/// of its deterministic per-row semantics whenever the input is typed.
pub proof fn lemma_apply_as_lift(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>,
    schema: Seq<VField>, output_schema: Seq<VField>,
)
    requires
        rows_in_schema(rows, schema),
        local_schema(plan, schema) == Some(output_schema),
    ensures
        apply_spec(plan, rows) == Ok(
            super::lifting::lift(
                rows,
                |r: Seq<Option<VValue>>| local_row_spec(plan, r).unwrap(),
            )),
    decreases rows.len(),
{
    lemma_apply_prefix_type_sound(plan, rows, schema, output_schema, rows.len());
    if rows.len() > 0 {
        let prefix = rows.drop_last();
        let tail = rows.last();
        lemma_rows_drop_last_valid(rows, schema);
        lemma_apply_as_lift(plan, prefix, schema, output_schema);
        lemma_apply_prefix_ignores_last(plan, rows, prefix.len());
        lemma_local_type_sound(plan, schema, tail, output_schema);
        assert(rows =~= prefix.push(tail));
        assert(apply_prefix_spec(plan, rows, prefix.len())
            == apply_spec(plan, prefix));
        match local_row_spec(plan, tail) {
            Ok(Some(_)) => { },
            Ok(None) => { },
            Err(_) => { assert(false); },
        }
    } else {
        assert(rows =~= Seq::empty());
    }
}

/// Every well-typed local node is a deterministic row-local lifting. A
/// single insertion can therefore add at most one output row, irrespective
/// of whether that node is a filter, projection, or another local operator.
pub proof fn lemma_apply_insert(
    plan: &VPlan, rows: Seq<Seq<Option<VValue>>>, schema: Seq<VField>,
    output_schema: Seq<VField>, index: int, row: Seq<Option<VValue>>,
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
{
    lemma_rows_insert_valid(rows, schema, index, row);
    lemma_apply_as_lift(plan, rows, schema, output_schema);
    lemma_apply_as_lift(plan, rows.insert(index, row), schema, output_schema);
    lemma_local_type_sound(plan, schema, row, output_schema);
    let f = |r: Seq<Option<VValue>>| local_row_spec(plan, r).unwrap();
    super::lifting::lemma_lift_insert(rows, index, row, f);
    let before = super::lifting::lift(rows, f);
    let after = super::lifting::lift(rows.insert(index, row), f);
    assert(apply_spec(plan, rows) == Ok(before));
    assert(apply_spec(plan, rows.insert(index, row)) == Ok(after));
    match local_row_spec(plan, row) {
        Ok(Some(added)) => {
            let i = choose|i: int|
                0 <= i <= before.len() && after =~= before.insert(i, added);
            assert(exists|j: int| 0 <= j <= before.len()
                && after =~= before.insert(j, added));
        },
        Ok(None) => {
            assert(after =~= before);
        },
        Err(_) => { assert(false); },
    }
}

} // verus!
