use crate::core::PrivacyMap;
use crate::measures::MaxDivergence;
use crate::metrics::DiscreteDistance;

use super::*;

/// counts accumulate per key, and keys keep the order they were first added in
#[test]
fn test_ordered_counts_groups_in_order() -> Fallible<()> {
    let mut counts = OrderedCounts::new();
    // enough keys that an unordered collection is unlikely to match by chance
    for key in ["e", "d", "c", "b", "a", "e"] {
        counts.add(key, 1)?;
    }
    counts.add("c", 3)?;

    let groups = counts.iter().cloned().collect::<Vec<_>>();
    assert_eq!(
        groups,
        vec![("e", 2), ("d", 1), ("c", 4), ("b", 1), ("a", 1)]
    );
    Ok(())
}

#[test]
fn test_ordered_counts_is_empty_before_any_add() {
    let counts = OrderedCounts::<u32>::new();
    assert_eq!(counts.iter().next(), None);
}

/// counts are checked, so a group can't silently wrap
#[test]
fn test_ordered_counts_detects_overflow() -> Fallible<()> {
    let mut counts = OrderedCounts::new();
    counts.add("a", u32::MAX)?;
    assert!(counts.add("a", 1).is_err());
    Ok(())
}

/// overflow can be detected before a group is charged
#[test]
fn test_ordered_counts_checks_without_adding() -> Fallible<()> {
    let mut counts = OrderedCounts::new();
    counts.add("a", u32::MAX)?;
    assert!(counts.check_add(&"a", 1).is_err());
    // a key that hasn't been seen starts from zero
    counts.check_add(&"b", u32::MAX)?;

    // checking leaves the groups untouched
    let groups = counts.iter().cloned().collect::<Vec<_>>();
    assert_eq!(groups, vec![("a", u32::MAX)]);
    Ok(())
}

/// privacy maps are grouped by identity, not by behavior
#[test]
fn test_ordered_counts_groups_privacy_maps_by_identity() -> Fallible<()> {
    let map = PrivacyMap::<DiscreteDistance, MaxDivergence>::new(|_| 1.0);
    let twin = PrivacyMap::<DiscreteDistance, MaxDivergence>::new(|_| 1.0);

    let mut counts = OrderedCounts::new();
    counts.add(map.clone(), 1)?;
    counts.add(map.clone(), 1)?;
    counts.add(twin, 1)?;

    // clones of one map share a group; an identical-but-separate map does not
    let ks = counts.iter().map(|(_, k)| *k).collect::<Vec<_>>();
    assert_eq!(ks, vec![2, 1]);
    Ok(())
}
