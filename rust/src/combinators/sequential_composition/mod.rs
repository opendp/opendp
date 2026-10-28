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

/// # Proof Definition
/// `composability` returns `Ok(out)` if the composition of a vector of privacy parameters `d_mids`
/// is bounded above by `self.compose(d_mids)` under `adaptivity` adaptivity and `out`-composability.
/// Otherwise returns an error.
/// Composition is commutative, so this bound holds for any ordering of `d_mids`.
///
/// Starting from `Self::Acc::default()` and charging privacy parameters one at a time via `self.accumulate`,
/// the composition of all privacy parameters charged to `acc` is bounded above by `self.compose_acc(acc)`.
pub trait CompositionMeasure: Measure {
    /// The state of a composition that is built up one privacy parameter at a time.
    /// The default is the empty composition.
    type Acc: Default;

    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composability>;
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance>;

    /// Charges the privacy parameter `eval()` of the privacy map identified by `key` to `acc` once.
    ///
    /// Every call on the same `acc` must evaluate the privacy map at the same `d_in`,
    /// as implementations may reuse the privacy parameter from an earlier charge of the same key.
    /// On error, the composition represented by `acc` is unchanged.
    fn accumulate(
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

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_MaxDivergence.tex"
)]
impl CompositionMeasure for MaxDivergence {
    type Acc = Self::Distance;

    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }

    fn accumulate(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = self.compose(vec![*acc, eval()?])?;
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
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }

    fn accumulate(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = self.compose(vec![*acc, eval()?])?;
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
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold((0.0, 0.0), |(eps_g, del_g), (eps_i, del_i)| {
                Ok((eps_g.inf_add(eps_i)?, del_g.inf_add(del_i)?))
            })
    }

    fn accumulate(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = self.compose(vec![*acc, eval()?])?;
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
    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_mids
            .iter()
            .try_fold((0.0, 0.0), |(eps_g, del_g), (eps_i, del_i)| {
                Ok((eps_g.inf_add(eps_i)?, del_g.inf_add(del_i)?))
            })
    }

    fn accumulate(
        &self,
        acc: &mut Self::Acc,
        _key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        *acc = self.compose(vec![*acc, eval()?])?;
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

    fn compose(&self, d_mids: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        Ok(Function::new_fallible(move |alpha| {
            d_mids
                .iter()
                .map(|f| f.eval(alpha))
                .try_fold(0.0, |sum, eps| sum.inf_add(&eps?))
        }))
    }

    fn accumulate(
        &self,
        acc: &mut Self::Acc,
        key: MapKey,
        eval: impl FnOnce() -> Fallible<Self::Distance>,
    ) -> Fallible<()> {
        let RenyiAcc { groups, indices } = acc;
        // charges of the same privacy map share a group,
        // so the map is evaluated once, not once per charge
        let i = match indices.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                groups.push((eval()?, 0));
                *entry.insert(groups.len() - 1)
            }
        };
        // a new group starts at zero, so only a charge to an existing group can overflow
        let k = &mut groups[i].1;
        *k = k.alerting_add(&1)?;
        Ok(())
    }

    fn compose_acc(&self, acc: &Self::Acc) -> Fallible<Self::Distance> {
        // O(m) in the number of distinct privacy maps
        let groups = acc.groups.clone();
        Ok(Function::new_fallible(move |alpha| {
            (groups.iter())
                .map(|(curve, k)| curve.eval(alpha)?.inf_mul(&f64::from(*k)))
                .try_fold(0.0, |sum, eps| sum.inf_add(&eps?))
        }))
    }
}

/// The state of a Rényi-DP composition:
/// the curve and number of charges of each distinct privacy map.
///
/// Charges are grouped by privacy map, so memory grows with the number of distinct privacy maps,
/// not the number of charges.
/// Groups are kept in the order their privacy maps were first charged,
/// so the composed curve sums in a consistent order.
#[derive(Default)]
pub struct RenyiAcc {
    groups: Vec<(Function<f64, f64>, u32)>,
    // index into groups by privacy map identity
    indices: HashMap<MapKey, usize>,
}
