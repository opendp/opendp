//! Logical semantics of grouped FrameDistance bounds.
//! These are independent of any particular transformation.

use super::*;
use vstd::multiset::Multiset;

verus! {

/// A finite, possibly overcomplete enumeration of row values. A distinct
/// enumeration of the joint support gives the usual per-group L1 distance.
pub open spec fn group_distance(
    left: Multiset<Seq<Option<VValue>>>,
    right: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    group: Seq<Option<VValue>>,
) -> nat
    decreases values.len(),
{
    if values.len() == 0 { 0 }
    else {
        let row = values.last();
        let a = left.count(row);
        let b = right.count(row);
        group_distance(left, right, values.drop_last(), group_by, group)
            + if group_by(row) == group {
                (if a >= b { a - b } else { b - a }) as nat
              } else { 0nat }
    }
}

/// Sum of indicators for changed groups. A distinct enumeration of all
/// possible group keys gives the `num_groups` component of FrameDistance.
pub open spec fn changed_groups(
    left: Multiset<Seq<Option<VValue>>>,
    right: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    groups: Seq<Seq<Option<VValue>>>,
) -> nat
    decreases groups.len(),
{
    if groups.len() == 0 { 0 }
    else {
        changed_groups(left, right, values, group_by, groups.drop_last())
            + if group_distance(left, right, values, group_by, groups.last()) > 0 {
                1nat
              } else { 0nat }
    }
}


/// Logical interpretation of one `Bound<VExpr>` once its grouping expressions
/// are interpreted as a deterministic row-local function. `values` must
/// enumerate the distinct joint row support and `groups` the distinct group
/// keys for this to equal the concrete FrameDistance components.
pub open spec fn within_group_bound(
    left: Multiset<Seq<Option<VValue>>>,
    right: Multiset<Seq<Option<VValue>>>,
    values: Seq<Seq<Option<VValue>>>,
    group_by: spec_fn(Seq<Option<VValue>>) -> Seq<Option<VValue>>,
    groups: Seq<Seq<Option<VValue>>>,
    per_group: Option<u32>, num_groups: Option<u32>,
) -> bool {
    &&& match per_group {
        Some(limit) => forall|key: Seq<Option<VValue>>|
            group_distance(left, right, values, group_by, key) <= limit as nat,
        None => true,
    }
    &&& match num_groups {
        Some(limit) => changed_groups(left, right, values, group_by, groups) <= limit as nat,
        None => true,
    }
}

} // verus!
