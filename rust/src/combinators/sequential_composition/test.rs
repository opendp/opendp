//! Invariants tested
//! 1. scalar accumulators match `compose` of the same privacy parameters
//! 2. a failed charge leaves the accumulator unchanged
//! 3. renyi charges group by privacy map identity, evaluating each map once
//! 4. a renyi read is a snapshot, unaffected by later charges
//! 5. renyi group counts can't silently overflow

use std::{cell::RefCell, rc::Rc};

use crate::core::PrivacyMap;
use crate::error::Fallible;
use crate::metrics::DiscreteDistance;

use super::*;

/// a privacy map that always returns `d_mid`
fn constant_map<MO: 'static + Measure>(d_mid: MO::Distance) -> PrivacyMap<DiscreteDistance, MO>
where
    MO::Distance: Clone,
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
    let composed = MaxDivergence.compose(d_mids.to_vec())?;
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
    assert!(acc.groups.is_empty());
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
    let ks = acc.groups.iter().map(|(_, k)| *k).collect::<Vec<_>>();
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

/// counts are checked, so a group can't silently wrap
#[test]
fn test_renyi_acc_detects_overflow() -> Fallible<()> {
    let map = constant_map::<RenyiDivergence>(Function::new(|alpha: &f64| 0.5 * alpha));

    let mut acc = RenyiDivergence.new_acc()?;
    RenyiDivergence.accumulate(&mut acc, &map, &1)?;
    // as if the map had already been charged u32::MAX times
    acc.groups[0].1 = u32::MAX;
    assert!(RenyiDivergence.accumulate(&mut acc, &map, &1).is_err());
    assert_eq!(acc.groups[0].1, u32::MAX);
    Ok(())
}
