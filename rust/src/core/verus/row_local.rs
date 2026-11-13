//! Generic multiset-stable lifting of a deterministic row-local operation.
//!
//! The operator is a total ghost function mapping an input element to either
//! one output element or nothing. This theorem is independent of VPlan,
//! Polars, domains, and the concrete element types.

use vstd::prelude::*;
use vstd::multiset::Multiset;

verus! {

/// Sequence realization of a deterministic map-or-drop operation.
pub open spec fn lift<X, Y>(
    rows: Seq<X>, f: spec_fn(X) -> Option<Y>,
) -> Seq<Y>
    decreases rows.len(),
{
    if rows.len() == 0 {
        Seq::<Y>::empty()
    } else {
        let prefix = lift(rows.drop_last(), f);
        match f(rows.last()) {
            Some(y) => prefix.push(y),
            None => prefix,
        }
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

/// A row-local operation maps one input insertion to either a single
/// output insertion or no output change. This also covers many-to-one
/// projections, so the theorem does not assume output values are distinct.
pub proof fn lemma_lift_insert<X, Y>(
    rows: Seq<X>, index: int, value: X, f: spec_fn(X) -> Option<Y>,
)
    requires 0 <= index <= rows.len(),
    ensures
        match f(value) {
            Some(y) => exists|j: int|
                0 <= j <= lift(rows, f).len()
                    && lift(rows.insert(index, value), f)
                        =~= lift(rows, f).insert(j, y),
            None => lift(rows.insert(index, value), f) =~= lift(rows, f),
        },
    decreases rows.len(),
{
    if index == rows.len() {
        assert(rows.insert(index, value) =~= rows.push(value));
        assert(rows.push(value).drop_last() =~= rows);
        match f(value) {
            Some(y) => {
                assert(lift(rows.insert(index, value), f) =~= lift(rows, f).push(y));
                assert(lift(rows, f).push(y)
                    =~= lift(rows, f).insert(lift(rows, f).len() as int, y));
            },
            None => {
                assert(lift(rows.insert(index, value), f) =~= lift(rows, f));
            },
        }
    } else {
        let prefix = rows.drop_last();
        let tail = rows.last();
        assert(rows =~= prefix.push(tail));
        assert(index <= prefix.len());
        assert(rows.insert(index, value) =~= prefix.insert(index, value).push(tail));
        assert(rows.insert(index, value).last() == tail);
        assert(rows.insert(index, value).drop_last()
            =~= prefix.insert(index, value));
        lemma_lift_insert(prefix, index, value, f);
        let before = lift(prefix, f);
        let after = lift(prefix.insert(index, value), f);
        assert(lift(rows, f)
            =~= match f(tail) { Some(y) => before.push(y), None => before });
        assert(lift(rows.insert(index, value), f)
            =~= match f(tail) { Some(y) => after.push(y), None => after });
        match f(value) {
            None => {
                assert(after =~= before);
            },
            Some(y) => {
                let j = choose|j: int|
                    0 <= j <= before.len() && after =~= before.insert(j, y);
                match f(tail) {
                    None => { },
                    Some(tail_output) => {
                        lemma_seq_insert_push(before, j, y, tail_output);
                        assert(j <= before.push(tail_output).len());
                        assert(after.push(tail_output)
                            =~= before.push(tail_output).insert(j, y));
                    },
                }
            },
        }
    }
}


/// Extensionality of lifting: only the mapping of elements actually present
/// in the input sequence can affect its result.
pub proof fn lemma_lift_equal_on_rows<X, Y>(
    rows: Seq<X>,
    f: spec_fn(X) -> Option<Y>,
    g: spec_fn(X) -> Option<Y>,
)
    requires forall|i: int| 0 <= i < rows.len() ==> f(rows[i]) == g(rows[i]),
    ensures lift(rows, f) =~= lift(rows, g),
    decreases rows.len(),
{
    if rows.len() > 0 {
        let prefix = rows.drop_last();
        assert forall|i: int| 0 <= i < prefix.len()
            implies f(prefix[i]) == g(prefix[i]) by {
            assert(prefix[i] == rows[i]);
        }
        lemma_lift_equal_on_rows(prefix, f, g);
        assert(f(rows.last()) == g(rows.last()));
    }
}

/// An element-preserving map-or-drop lift is exactly multiset filtering.
pub open spec fn filter_lift<X>(
    rows: Seq<X>, keep: spec_fn(X) -> bool,
) -> Seq<X> {
    lift(rows, |r: X| if keep(r) { Some(r) } else { None })
}

pub proof fn lemma_filter_lift_multiset<X>(
    rows: Seq<X>, keep: spec_fn(X) -> bool,
)
    ensures
        filter_lift(rows, keep).to_multiset()
            =~= rows.to_multiset().filter(keep),
    decreases rows.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if rows.len() == 0 {
        rows.to_multiset_ensures();
        filter_lift(rows, keep).to_multiset_ensures();
        assert(rows.to_multiset() =~= Multiset::<X>::empty());
        assert(filter_lift(rows, keep).to_multiset()
            =~= Multiset::<X>::empty());
    } else {
        let prefix = rows.drop_last();
        let last = rows.last();
        lemma_filter_lift_multiset(prefix, keep);
        assert(rows =~= prefix.push(last));
        let before = filter_lift(prefix, keep);
        let after = filter_lift(rows, keep);
        prefix.to_multiset_ensures();
        rows.to_multiset_ensures();
        before.to_multiset_ensures();
        if keep(last) {
            assert(after =~= before.push(last));
            after.to_multiset_ensures();
        } else {
            assert(after =~= before);
        }
        assert forall|value: X|
            after.to_multiset().count(value)
                == rows.to_multiset().filter(keep).count(value) by {
            if value == last {
            } else {
            }
        }
    }
}

/// A deterministic map-or-drop operator never produces more output rows
/// than there are input rows, counting duplicates.
pub proof fn lemma_lift_length<X, Y>(
    rows: Seq<X>, f: spec_fn(X) -> Option<Y>,
)
    ensures lift(rows, f).to_multiset().len() <= rows.to_multiset().len(),
    decreases rows.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if rows.len() > 0 {
        let prefix = rows.drop_last();
        lemma_lift_length(prefix, f);
        assert(rows =~= prefix.push(rows.last()));
        prefix.to_multiset_ensures();
        let before = lift(prefix, f);
        before.to_multiset_ensures();
        match f(rows.last()) {
            Some(value) => {
                assert(lift(rows, f) =~= before.push(value));
                assert(lift(rows, f).to_multiset().len()
                    == before.to_multiset().len() + 1);
            },
            None => {
                assert(lift(rows, f) =~= before);
            },
        }
        rows.to_multiset_ensures();
        assert(rows.to_multiset().len() == prefix.to_multiset().len() + 1);
    } else {
        rows.to_multiset_ensures();
        lift(rows, f).to_multiset_ensures();
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
            Some(y) => lift(rows.insert(index, value), f).to_multiset()
                =~= lift(rows, f).to_multiset().insert(y),
            None => lift(rows.insert(index, value), f).to_multiset()
                =~= lift(rows, f).to_multiset(),
        },
{
    lemma_lift_insert(rows, index, value, f);
    if let Some(y) = f(value) {
        let before = lift(rows, f);
        let after = lift(rows.insert(index, value), f);
        let j = choose|j: int| 0 <= j <= before.len()
            && after =~= before.insert(j, y);
        lemma_seq_insert_bag(before, j, y);
    }
}

} // verus!
