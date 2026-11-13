//! Shared lifting lemmas for typed row-local operators and arbitrary symmetric distance.

use super::*;

verus! {

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
            lifting::lift(
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

} // verus!
