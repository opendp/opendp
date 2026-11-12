//! Shared lifting lemmas for typed row-local operators and arbitrary symmetric distance.

use super::*;
use super::super::verified::symmetric_distance as metric;
use vstd::multiset::Multiset;

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

/// A deterministic map-or-drop operator never produces more output rows
/// than there are input rows, counting duplicates.
pub proof fn lemma_lift_length<X, Y>(
    rows: Seq<X>, f: spec_fn(X) -> Option<Y>,
)
    ensures lifting::lift(rows, f).to_multiset().len() <= rows.to_multiset().len(),
    decreases rows.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if rows.len() > 0 {
        let prefix = rows.drop_last();
        lemma_lift_length(prefix, f);
        assert(rows =~= prefix.push(rows.last()));
        prefix.to_multiset_ensures();
        let before = lifting::lift(prefix, f);
        before.to_multiset_ensures();
        match f(rows.last()) {
            Some(value) => {
                assert(lifting::lift(rows, f) =~= before.push(value));
                assert(lifting::lift(rows, f).to_multiset().len()
                    == before.to_multiset().len() + 1);
            },
            None => {
                assert(lifting::lift(rows, f) =~= before);
            },
        }
        rows.to_multiset_ensures();
        assert(rows.to_multiset().len() == prefix.to_multiset().len() + 1);
    } else {
        rows.to_multiset_ensures();
        lifting::lift(rows, f).to_multiset_ensures();
    }
}

/// Inserting a row anywhere in a sequence inserts one copy in its multiset.
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

/// A sequence insertion corresponds to either one output multiset
/// insertion or no change. This is valid even for many-to-one maps.
pub proof fn lemma_lift_bag_insert<X, Y>(
    rows: Seq<X>, index: int, value: X, f: spec_fn(X) -> Option<Y>,
)
    requires 0 <= index <= rows.len(),
    ensures
        match f(value) {
            Some(y) => lifting::lift(rows.insert(index, value), f).to_multiset()
                =~= lifting::lift(rows, f).to_multiset().insert(y),
            None => lifting::lift(rows.insert(index, value), f).to_multiset()
                =~= lifting::lift(rows, f).to_multiset(),
        },
{
    lifting::lemma_lift_insert(rows, index, value, f);
    if let Some(y) = f(value) {
        let before = lifting::lift(rows, f);
        let after = lifting::lift(rows.insert(index, value), f);
        let j = choose|j: int| 0 <= j <= before.len()
            && after =~= before.insert(j, y);
        lemma_seq_insert_bag(before, j, y);
    }
}

/// Every deterministic row-local map-or-drop is nonexpansive under
/// symmetric multiset distance. The proof cancels matching input copies;
/// each remaining insertion or deletion can change the output by at most
/// one. No injectivity or row preservation assumption is needed.
pub proof fn lemma_lift_contraction<X, Y>(
    left: Seq<X>, right: Seq<X>, f: spec_fn(X) -> Option<Y>,
)
    ensures
        metric::distance(
            lifting::lift(left, f).to_multiset(),
            lifting::lift(right, f).to_multiset(),
        ) <= metric::distance(
            left.to_multiset(), right.to_multiset(),
        ),
    decreases left.len() + right.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if left.len() == 0 {
        assert(left =~= Seq::<X>::empty());
        assert(lifting::lift(left, f) =~= Seq::<Y>::empty());
        left.to_multiset_ensures();
        lifting::lift(left, f).to_multiset_ensures();
        assert(left.to_multiset() =~= Multiset::<X>::empty());
        assert(lifting::lift(left, f).to_multiset() =~= Multiset::<Y>::empty());
        lemma_lift_length(right, f);
        metric::lemma_empty_distance(right.to_multiset());
        metric::lemma_empty_distance(lifting::lift(right, f).to_multiset());
    } else {
        let prefix = left.drop_last();
        let value = left.last();
        assert(left =~= prefix.push(value));
        prefix.to_multiset_ensures();
        left.to_multiset_ensures();
        let left_before = lifting::lift(prefix, f).to_multiset();
        let left_after = lifting::lift(left, f).to_multiset();
        lifting::lift(prefix, f).to_multiset_ensures();
        if right.to_multiset().count(value) > 0 {
            right.to_multiset_ensures();
            assert(right.contains(value));
            let index = choose|i: int| 0 <= i < right.len() && right[i] == value;
            let reduced = right.remove(index);
            right.remove_ensures(index);
            assert(reduced.len() + 1 == right.len());
            assert(0 <= index <= reduced.len());
            reduced.insert_ensures(index, value);
            let restored = reduced.insert(index, value);
            assert(restored.len() == right.len());
            assert forall|i: int| 0 <= i < right.len()
                implies right[i] == restored[i] by {
                if i < index {
                } else if i == index {
                } else {
                }
            }
            assert(right =~= restored);
            lemma_seq_insert_bag(reduced, index, value);
            assert(right.to_multiset() =~= reduced.to_multiset().insert(value));
            lemma_lift_contraction(prefix, reduced, f);
            lemma_lift_bag_insert(reduced, index, value, f);
            assert(left.to_multiset() =~= prefix.to_multiset().insert(value));
            let right_before = lifting::lift(reduced, f).to_multiset();
            let right_after = lifting::lift(right, f).to_multiset();
            match f(value) {
                Some(y) => {
                    assert(left_after =~= left_before.insert(y));
                    assert(right_after =~= right_before.insert(y));
                    metric::lemma_insert_common(left_before, right_before, y);
                },
                None => {
                    assert(left_after =~= left_before);
                    assert(right_after =~= right_before);
                },
            }
            metric::lemma_insert_common(
                prefix.to_multiset(), reduced.to_multiset(), value);
        } else {
            lemma_lift_contraction(prefix, right, f);
            assert(left.to_multiset() =~= prefix.to_multiset().insert(value));
            metric::lemma_insert_fresh(prefix.to_multiset(), right.to_multiset(), value);
            match f(value) {
                Some(y) => {
                    assert(left_after =~= left_before.insert(y));
                    metric::lemma_insert_bound(
                        left_before, lifting::lift(right, f).to_multiset(), y);
                },
                None => {
                    assert(left_after =~= left_before);
                },
            }
        }
    }
}

} // verus!
