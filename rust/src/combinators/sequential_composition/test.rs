use std::{cell::RefCell, rc::Rc};

use crate::core::PrivacyMap;
use crate::error::Fallible;
use crate::metrics::DiscreteDistance;

use super::*;

/// counts accumulate per key, and keys keep the order they were first added in
#[test]
fn test_ordered_counts_groups_in_order() -> Fallible<()> {
    let mut counts = OrderedCounts::new();
    // enough keys that an unordered collection is unlikely to match by chance
    for key in ["e", "d", "c", "b", "a", "e"] {
        counts.add(key, 1, || Ok(key))?;
    }
    counts.add("c", 3, || Ok("c"))?;

    let groups = counts.iter().cloned().collect::<Vec<_>>();
    assert_eq!(
        groups,
        vec![("e", 2), ("d", 1), ("c", 4), ("b", 1), ("a", 1)]
    );
    Ok(())
}

#[test]
fn test_ordered_counts_is_empty_before_any_add() {
    let counts = OrderedCounts::<u32, u32>::new();
    assert_eq!(counts.iter().next(), None);
}

/// the value of a group is computed when the group starts, and never again
#[test]
fn test_ordered_counts_computes_value_once() -> Fallible<()> {
    let mut counts = OrderedCounts::new();
    counts.add("a", 1, || Ok(1))?;
    counts.add("a", 1, || -> Fallible<i32> { panic!("value recomputed") })?;
    // a failed value starts no group
    assert!(
        counts
            .add("b", 1, || fallible!(FailedFunction, "no value"))
            .is_err()
    );

    assert_eq!(counts.iter().cloned().collect::<Vec<_>>(), vec![(1, 2)]);
    Ok(())
}

/// counts are checked, so a group can't silently wrap
#[test]
fn test_ordered_counts_detects_overflow() -> Fallible<()> {
    let mut counts = OrderedCounts::new();
    counts.add("a", u32::MAX, || Ok(()))?;
    assert!(counts.add("a", 1, || Ok(())).is_err());
    assert_eq!(counts.iter().next(), Some(&((), u32::MAX)));
    Ok(())
}

/// a privacy map that always returns `d_mid`
fn constant_map<MO: 'static + Measure>(d_mid: MO::Distance) -> PrivacyMap<DiscreteDistance, MO>
where
    MO::Distance: 'static + Clone,
{
    PrivacyMap::new(move |_| d_mid.clone())
}

/// accumulating one privacy parameter at a time matches composing them all at once
#[test]
fn test_scalar_acc_matches_compose() -> Fallible<()> {
    let d_mids = [0.1, 0.2, 0.3, 0.1];

    let mut acc = MaxDivergence.new_acc()?;
    for d_mid in d_mids {
        MaxDivergence.accumulate(&mut acc, &constant_map(d_mid), &1)?;
    }
    let composed = MaxDivergence.compose(d_mids.iter().map(|d| (*d, 1)).collect())?;
    assert_eq!(MaxDivergence.compose_acc(&acc)?, composed);
    Ok(())
}

/// a failed charge leaves the accumulator unchanged, even when only one coordinate overflows
#[test]
fn test_approximate_acc_unchanged_on_error() -> Fallible<()> {
    let measure = Approximate(MaxDivergence);
    let map = constant_map((1.0, f64::MAX));

    let mut acc = measure.new_acc()?;
    measure.accumulate(&mut acc, &map, &1)?;
    // epsilon would add fine, but delta overflows
    assert!(measure.accumulate(&mut acc, &map, &1).is_err());
    assert_eq!(measure.compose_acc(&acc)?, (1.0, f64::MAX));
    Ok(())
}

/// a privacy map that fails leaves the accumulator unchanged
#[test]
fn test_renyi_acc_unchanged_on_failing_map() -> Fallible<()> {
    let failing = PrivacyMap::<DiscreteDistance, RenyiDivergence>::new_fallible(|_| {
        fallible!(FailedMap, "map fails")
    });

    let mut acc = RenyiDivergence.new_acc()?;
    assert!(RenyiDivergence.accumulate(&mut acc, &failing, &1).is_err());
    assert_eq!(acc.0.iter().count(), 0);
    Ok(())
}

/// the renyi accumulator groups charges by privacy map identity, not by behavior,
/// and evaluates each privacy map once
#[test]
fn test_renyi_acc_groups_by_map_identity() -> Fallible<()> {
    let evals = Rc::new(RefCell::new(0));
    // allocates a new curve on every evaluation, as privacy maps over FFI do
    let map = PrivacyMap::<DiscreteDistance, RenyiDivergence>::new(enclose!(evals, move |_| {
        *evals.borrow_mut() += 1;
        Function::new(|alpha: &f64| 0.5 * alpha)
    }));
    let twin = PrivacyMap::<DiscreteDistance, RenyiDivergence>::new(|_| {
        Function::new(|alpha: &f64| 0.5 * alpha)
    });

    let mut acc = RenyiDivergence.new_acc()?;
    for _ in 0..3 {
        RenyiDivergence.accumulate(&mut acc, &map, &1)?;
    }
    RenyiDivergence.accumulate(&mut acc, &twin, &1)?;

    // clones of one map share a group; an identical-but-separate map does not
    let ks = acc.0.iter().map(|(_, k)| *k).collect::<Vec<_>>();
    assert_eq!(ks, vec![3, 1]);
    assert_eq!(*evals.borrow(), 1);
    assert_eq!(RenyiDivergence.compose_acc(&acc)?.eval(&2.0)?, 4.0);
    Ok(())
}

/// reading the renyi loss leaves the accumulator open to more charges
#[test]
fn test_renyi_acc_snapshot_is_independent() -> Fallible<()> {
    let map = constant_map::<RenyiDivergence>(Function::new(|alpha: &f64| 0.5 * alpha));

    let mut acc = RenyiDivergence.new_acc()?;
    RenyiDivergence.accumulate(&mut acc, &map, &1)?;
    let before = RenyiDivergence.compose_acc(&acc)?;
    RenyiDivergence.accumulate(&mut acc, &map, &1)?;

    assert_eq!(before.eval(&2.0)?, 1.0);
    assert_eq!(RenyiDivergence.compose_acc(&acc)?.eval(&2.0)?, 2.0);
    Ok(())
}
