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

} // verus!
