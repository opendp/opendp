//! Verus semantics for OpenDP's SymmetricDistance.
//!
//! A dataset is interpreted as a multiset, independent of row ordering.
//! Its distance is the number of unmatched copies after cancellation:
//! |left \ right| + |right \ left|. This is a natural number; the public
//! IntDistance (u32) is an upper bound, not the range of the exact distance.
//! Using a natural number avoids overflow when the two datasets are large.

use vstd::prelude::*;
use vstd::multiset::Multiset;

verus! {

/// Exact symmetric multiset distance, counting additions and removals.
pub open spec fn distance<T>(left: Multiset<T>, right: Multiset<T>) -> nat {
    left.sub(right).len() + right.sub(left).len()
}

/// The d-closeness relation of OpenDP's SymmetricDistance, for IntDistance=u32.
pub open spec fn close<T>(
    left: Multiset<T>, right: Multiset<T>, d: u32,
) -> bool {
    distance(left, right) <= d as nat
}

/// Multiset inclusion cannot increase total multiplicity.
pub proof fn lemma_subset_length<T>(
    small: Multiset<T>, big: Multiset<T>,
)
    requires small.subset_of(big),
    ensures small.len() <= big.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    let remainder = big.sub(small);
    assert forall|v: T| small.add(remainder).count(v) == big.count(v) by {
        assert(small.count(v) <= big.count(v));
    }
    assert(small.add(remainder) =~= big);
    assert(small.add(remainder).len() == small.len() + remainder.len());
}

/// Symmetric multiset distance is a metric on multisets (and a
/// pseudometric on ordered vectors, because row order is ignored).
pub proof fn lemma_reflexive<T>(a: Multiset<T>)
    ensures distance(a, a) == 0,
{
    broadcast use vstd::multiset::group_multiset_axioms;
    assert forall|v: T| a.sub(a).count(v) == 0 by { }
    assert(a.sub(a) =~= Multiset::<T>::empty());
    assert(a.sub(a).len() == 0);
}

pub proof fn lemma_symmetric<T>(a: Multiset<T>, b: Multiset<T>)
    ensures distance(a, b) == distance(b, a),
{
}

pub proof fn lemma_separating<T>(a: Multiset<T>, b: Multiset<T>)
    ensures (distance(a, b) == 0) <==> (a =~= b),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    if a =~= b {
        lemma_reflexive(a);
    } else if distance(a, b) == 0 {
        assert(a.sub(b).len() == 0);
        assert(b.sub(a).len() == 0);
        assert(a.sub(b) =~= Multiset::<T>::empty());
        assert(b.sub(a) =~= Multiset::<T>::empty());
        assert forall|v: T| a.count(v) == b.count(v) by {
            assert(a.sub(b).count(v) == 0);
            assert(b.sub(a).count(v) == 0);
        }
        assert(a =~= b);
    }
}

pub proof fn lemma_triangle<T>(
    a: Multiset<T>, b: Multiset<T>, c: Multiset<T>,
)
    ensures distance(a, c) <= distance(a, b) + distance(b, c),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    assert forall|v: T|
        a.sub(c).count(v) <= a.sub(b).add(b.sub(c)).count(v) by {
        let x = a.count(v);
        let y = b.count(v);
        let z = c.count(v);
        if x > z {
            if x >= y {
                if y >= z { } else { }
            } else {
                assert(y > z);
            }
        }
    }
    assert forall|v: T|
        c.sub(a).count(v) <= c.sub(b).add(b.sub(a)).count(v) by {
        let x = c.count(v);
        let y = b.count(v);
        let z = a.count(v);
        if x > z {
            if x >= y {
                if y >= z { } else { }
            } else {
                assert(y > z);
            }
        }
    }
    lemma_subset_length(a.sub(c), a.sub(b).add(b.sub(c)));
    lemma_subset_length(c.sub(a), c.sub(b).add(b.sub(a)));
    assert(a.sub(b).add(b.sub(c)).len()
        == a.sub(b).len() + b.sub(c).len());
    assert(c.sub(b).add(b.sub(a)).len()
        == c.sub(b).len() + b.sub(a).len());
}

/// Filtering is contractive under the exact symmetric multiset distance.
///
/// This lemma is independent of the element type and filtering predicate.
/// It is the generic metric law reused by verified row-filter constructors.
pub proof fn lemma_filter_contraction<T>(
    left: Multiset<T>,
    right: Multiset<T>,
    keep: spec_fn(T) -> bool,
)
    ensures distance(left.filter(keep), right.filter(keep)) <= distance(left, right),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    let left_only = left.sub(right);
    let right_only = right.sub(left);
    let kept_left_only = left.filter(keep).sub(right.filter(keep));
    let kept_right_only = right.filter(keep).sub(left.filter(keep));

    assert forall|v: T|
        kept_left_only.count(v) <= left_only.count(v) by {
        if keep(v) {
            assert(left.filter(keep).count(v) == left.count(v));
            assert(right.filter(keep).count(v) == right.count(v));
        } else {
            assert(left.filter(keep).count(v) == 0);
        }
    }
    assert forall|v: T|
        kept_right_only.count(v) <= right_only.count(v) by {
        if keep(v) {
            assert(left.filter(keep).count(v) == left.count(v));
            assert(right.filter(keep).count(v) == right.count(v));
        } else {
            assert(right.filter(keep).count(v) == 0);
        }
    }
    lemma_subset_length(kept_left_only, left_only);
    lemma_subset_length(kept_right_only, right_only);
}


/** Adding the same row on both sides leaves symmetric distance unchanged. */
pub proof fn lemma_insert_common<T>(
    left: Multiset<T>, right: Multiset<T>, value: T,
)
    ensures distance(left.insert(value), right.insert(value)) == distance(left, right),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    assert forall|v: T|
        left.insert(value).sub(right.insert(value)).count(v)
            == left.sub(right).count(v) by { }
    assert forall|v: T|
        right.insert(value).sub(left.insert(value)).count(v)
            == right.sub(left).count(v) by { }
    assert(left.insert(value).sub(right.insert(value)) =~= left.sub(right));
    assert(right.insert(value).sub(left.insert(value)) =~= right.sub(left));
}

/// If the other dataset has no copy of a row, inserting one increases
/// the symmetric distance by exactly one.
pub proof fn lemma_insert_fresh<T>(
    left: Multiset<T>, right: Multiset<T>, value: T,
)
    requires right.count(value) == 0,
    ensures distance(left.insert(value), right) == distance(left, right) + 1,
{
    broadcast use vstd::multiset::group_multiset_axioms;
    assert forall|v: T|
        left.insert(value).sub(right).count(v)
            == left.sub(right).insert(value).count(v) by {
        if v == value { assert(right.count(v) == 0); }
    }
    assert forall|v: T|
        right.sub(left.insert(value)).count(v)
            == right.sub(left).count(v) by {
        if v == value { assert(right.count(v) == 0); }
    }
    assert(left.insert(value).sub(right) =~= left.sub(right).insert(value));
    assert(right.sub(left.insert(value)) =~= right.sub(left));
    assert(left.sub(right).insert(value).len() == left.sub(right).len() + 1);
}

/// Adding one row to only one dataset changes its symmetric distance
/// from the other dataset by at most one.
pub proof fn lemma_insert_bound<T>(
    left: Multiset<T>, right: Multiset<T>, value: T,
)
    ensures distance(left.insert(value), right) <= distance(left, right) + 1,
{
    broadcast use vstd::multiset::group_multiset_axioms;
    assert forall|v: T|
        left.insert(value).sub(left).count(v)
            == Multiset::<T>::singleton(value).count(v) by { }
    assert forall|v: T|
        left.sub(left.insert(value)).count(v) == 0 by { }
    assert(left.insert(value).sub(left) =~= Multiset::<T>::singleton(value));
    assert(left.sub(left.insert(value)) =~= Multiset::<T>::empty());
    assert(distance(left.insert(value), left) == 1);
    lemma_triangle(left.insert(value), left, right);
}

/// The distance from the empty dataset is its total multiplicity.
pub proof fn lemma_empty_distance<T>(rows: Multiset<T>)
    ensures
        distance(Multiset::<T>::empty(), rows) == rows.len(),
        distance(rows, Multiset::<T>::empty()) == rows.len(),
{
    broadcast use vstd::multiset::group_multiset_axioms;
    assert forall|v: T|
        rows.sub(Multiset::<T>::empty()).count(v) == rows.count(v) by { }
    assert forall|v: T|
        Multiset::<T>::empty().sub(rows).count(v) == 0 by { }
    assert(rows.sub(Multiset::<T>::empty()) =~= rows);
    assert(Multiset::<T>::empty().sub(rows) =~= Multiset::<T>::empty());
}

} // verus!
