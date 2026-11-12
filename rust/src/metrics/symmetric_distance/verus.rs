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

} // verus!
