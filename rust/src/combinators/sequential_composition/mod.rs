use std::any::Any;
use std::collections::{HashMap, hash_map::Entry};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[cfg(feature = "contrib")]
mod non_adaptive;
#[cfg(feature = "contrib")]
pub use non_adaptive::*;

#[cfg(feature = "contrib")]
mod adaptive;
#[cfg(feature = "contrib")]
pub use adaptive::*;

#[cfg(feature = "contrib")]
mod fully_adaptive;
#[cfg(feature = "contrib")]
pub use fully_adaptive::*;
use opendp_derive::proven;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

use crate::{
    core::{Function, Measure, Metric, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
    traits::{AlertingAdd, InfAdd, InfMul},
};

#[derive(Debug)]
pub enum Adaptivity {
    /// All queries are executed together in a single batch.
    NonAdaptive,
    /// The privacy loss parameters are non-adaptive,
    /// but the queries can be chosen adaptively based on the results of previous queries.
    Adaptive,
    /// The privacy loss parameters and queries can be chosen adaptively
    /// based on the results of previous queries.
    FullyAdaptive,
}

#[derive(Debug)]
pub enum Composability {
    /// Previous interactive mechanisms are locked when a new query is submitted.
    Sequential,
    /// Previous interactive mechanisms are not locked when a new query is submitted.
    Concurrent,
}

/// `(value, k)` groups, indexed by key, in the order keys are first added.
///
/// We use a Vec for order and HashMap for o(1) lookup.
/// Order improves consistency for testing with an o(n) memory cost.
pub(crate) struct OrderedCounts<K: Eq + Hash, V> {
    groups: Vec<(V, u32)>,
    // index into groups by key identity
    indices: HashMap<K, usize>,
}

impl<K: Eq + Hash, V> OrderedCounts<K, V> {
    pub fn new() -> Self {
        OrderedCounts {
            groups: Vec::new(),
            indices: HashMap::new(),
        }
    }

    /// charge `k` to the group for `key`, starting a new group with `value()` if there isn't one
    ///
    /// On error, the counts are unchanged.
    pub fn add(&mut self, key: K, k: u32, value: impl FnOnce() -> Fallible<V>) -> Fallible<()> {
        let OrderedCounts { groups, indices } = self;
        let i = match indices.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                groups.push((value()?, 0));
                *entry.insert(groups.len() - 1)
            }
        };
        // a new group starts at zero, so only a charge to an existing group can overflow
        let count = &mut groups[i].1;
        *count = count.alerting_add(&k)?;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &(V, u32)> {
        self.groups.iter()
    }
}

/// # Proof Definition
/// `composability` returns `Ok(out)` if the composition of a vector of privacy parameters `d_mids`,
/// where each parameter `d_mid_i` is charged with multiplicity `k_i`,
/// is bounded above by `self.compose(d_mids)` under `adaptivity` adaptivity and `out`-composability.
/// Otherwise returns an error.
/// Composition is commutative, so this bound holds for any ordering of `d_mids`.
///
/// Starting from `self.new_acc()` and charging privacy parameters one at a time via `self.accumulate_keyed`,
/// `self.compose_acc(acc)` is bounded above by the composition of all privacy parameters charged to `acc`.
pub trait CompositionMeasure: Measure {
    /// The state of a composition that is built up one privacy parameter at a time.
    type Acc;

    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composability>;
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance>;

    /// Returns the state of a composition of no privacy parameters.
    fn new_acc(&self) -> Fallible<Self::Acc>;

    /// Charges the privacy parameter of `map` at `d_in` to `acc` once.
    ///
    /// Every call on the same `acc` must use the same `d_in`,
    /// as implementations may reuse the privacy parameter from an earlier charge of the same `map`.
    /// On error, the composition represented by `acc` is unchanged.
    fn accumulate<MI: 'static + Metric>(
        &self,
        acc: &mut Self::Acc,
        map: &PrivacyMap<MI, Self>,
        d_in: &MI::Distance,
    ) -> Fallible<()>
    where
        Self: 'static,
    {
        self.accumulate_keyed(acc, MapKey::new(map), || map.eval(d_in))
    }

    /// Charges the privacy parameter `eval()` of the privacy map identified by `key` to `acc` once.
    ///
    /// `MapKey`s can only be constructed in this crate, so call `accumulate` instead.
    /// On error, the composition represented by `acc` is unchanged.
    fn accumulate_keyed(
        &self,
        acc: &mut Self::Acc,
        key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()>;

    /// Returns the composition of all privacy parameters charged to `acc`.
    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance>;
}

/// Identifies a privacy map by its allocation.
///
/// The key keeps the privacy map alive, so the allocation is not reused by another privacy map
/// while the key exists.
pub struct MapKey {
    ptr: *const (),
    _map: Box<dyn Any>,
}

impl MapKey {
    pub(crate) fn new<MI: 'static + Metric, MO: 'static + Measure>(
        map: &PrivacyMap<MI, MO>,
    ) -> Self {
        MapKey {
            ptr: Arc::as_ptr(&map.0) as *const (),
            _map: Box::new(map.clone()),
        }
    }
}

impl PartialEq for MapKey {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}
impl Eq for MapKey {}

impl Hash for MapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state)
    }
}

/// A privacy parameter that composes by addition.
///
/// Additive measures don't need to remember individual charges,
/// so their composition state is just the running total.
pub(crate) trait AdditiveDistance: Copy {
    /// The privacy parameter of a composition of no queries.
    const ZERO: Self;

    /// Returns `self + d * k`, rounded up.
    ///
    /// Returns a new value, so `self` is unchanged on error.
    fn add_scaled(&self, d: &Self, k: u32) -> Fallible<Self>;
}

impl AdditiveDistance for f64 {
    const ZERO: Self = 0.0;
    fn add_scaled(&self, d: &Self, k: u32) -> Fallible<Self> {
        self.inf_add(&d.inf_mul(&f64::from(k))?)
    }
}

impl AdditiveDistance for (f64, f64) {
    const ZERO: Self = (0.0, 0.0);
    fn add_scaled(&self, d: &Self, k: u32) -> Fallible<Self> {
        // each component composes independently
        Ok((self.0.add_scaled(&d.0, k)?, self.1.add_scaled(&d.1, k)?))
    }
}

/// Sums `d_mids`, each `d_i` charged `k_i` times.
fn compose_additive<D: AdditiveDistance>(d_mids: Vec<(D, u32)>) -> Fallible<D> {
    (d_mids.iter()).try_fold(D::ZERO, |sum, (d_i, k_i)| sum.add_scaled(d_i, *k_i))
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_MaxDivergence.tex"
)]
impl CompositionMeasure for MaxDivergence {
    type Acc = Self::Distance;

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        compose_additive(d_mids)
    }

    fn new_acc(&self) -> Fallible<Self::Acc> {
        Ok(AdditiveDistance::ZERO)
    }
    fn accumulate_keyed(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = acc.add_scaled(&eval()?, 1)?;
        Ok(())
    }
    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        Ok(*acc)
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ZeroConcentratedDivergence.tex"
)]
impl CompositionMeasure for ZeroConcentratedDivergence {
    type Acc = Self::Distance;

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        compose_additive(d_mids)
    }

    fn new_acc(&self) -> Fallible<Self::Acc> {
        Ok(AdditiveDistance::ZERO)
    }
    fn accumulate_keyed(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = acc.add_scaled(&eval()?, 1)?;
        Ok(())
    }
    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        Ok(*acc)
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproximateMaxDivergence.tex"
)]
impl CompositionMeasure for Approximate<MaxDivergence> {
    type Acc = Self::Distance;

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        compose_additive(d_mids)
    }

    fn new_acc(&self) -> Fallible<Self::Acc> {
        Ok(AdditiveDistance::ZERO)
    }
    fn accumulate_keyed(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = acc.add_scaled(&eval()?, 1)?;
        Ok(())
    }
    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        Ok(*acc)
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproximateZeroConcentratedDivergence.tex"
)]
impl CompositionMeasure for Approximate<ZeroConcentratedDivergence> {
    type Acc = Self::Distance;

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Sequential)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        compose_additive(d_mids)
    }

    fn new_acc(&self) -> Fallible<Self::Acc> {
        Ok(AdditiveDistance::ZERO)
    }
    fn accumulate_keyed(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = acc.add_scaled(&eval()?, 1)?;
        Ok(())
    }
    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        Ok(*acc)
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_RenyiDivergence.tex"
)]
impl CompositionMeasure for RenyiDivergence {
    type Acc = RenyiAcc;

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }

    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        // merge equal curves so that each is evaluated once, not once per copy
        let mut groups = OrderedCounts::new();
        for (d_mid, k_i) in d_mids {
            groups.add(d_mid.clone(), k_i, || Ok(d_mid))?;
        }
        Ok(Function::new_fallible(move |alpha| {
            (groups.iter())
                .map(|(curve, k)| curve.eval(alpha)?.inf_mul(&f64::from(*k)))
                .try_fold(0.0, |sum, eps| sum.inf_add(&eps?))
        }))
    }

    fn new_acc(&self) -> Fallible<Self::Acc> {
        Ok(RenyiAcc(OrderedCounts::new()))
    }

    fn accumulate_keyed(
        &self,
        acc: &mut Self::Acc,
        key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        // charges of the same privacy map share a group,
        // so the map is evaluated once, not once per charge
        acc.0.add(key, 1, eval)
    }

    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        // O(m) in the number of distinct privacy maps
        self.compose(acc.0.iter().cloned().collect())
    }
}

/// The state of a Rényi-DP composition:
/// the curve and number of charges of each distinct privacy map.
///
/// Charges are grouped by privacy map, so memory grows with the number of distinct privacy maps,
/// not the number of charges.
pub struct RenyiAcc(OrderedCounts<MapKey, Function<f64, f64>>);
