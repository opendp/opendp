use std::collections::HashMap;
use std::hash::Hash;

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
    core::{Function, Measure},
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

/// `(key, k)` groups in the order keys are first added.
///
/// We use a Vec for order and HashMap for o(1) lookup.
/// Order improves consistency for testing with an o(n) memory cost.
pub(crate) struct OrderedCounts<K: Clone + Eq + Hash> {
    groups: Vec<(K, u32)>,
    // index into groups by key identity
    indices: HashMap<K, usize>,
}

impl<K: Clone + Eq + Hash> OrderedCounts<K> {
    pub fn new() -> Self {
        OrderedCounts {
            groups: Vec::new(),
            indices: HashMap::new(),
        }
    }

    /// charge `k` to the group for `key`, starting a new group if there isn't one
    pub fn add(&mut self, key: K, k: u32) -> Fallible<()> {
        let OrderedCounts { groups, indices } = self;
        let i = *(indices.entry(key.clone())).or_insert_with(|| {
            groups.push((key, 0));
            groups.len() - 1
        });
        let count = &mut groups[i].1;
        *count = count.alerting_add(&k)?;
        Ok(())
    }

    /// error if charging `k` to the group for `key` would overflow
    pub fn check_add(&self, key: &K, k: u32) -> Fallible<()> {
        let count = (self.indices.get(key)).map_or(0, |&i| self.groups[i].1);
        count.alerting_add(&k).map(|_| ())
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, u32)> {
        self.groups.iter()
    }
}

/// # Proof Definition
/// `composability` returns `Ok(out)` if the composition of a vector of privacy parameters `d_mids`,
/// where each parameter `d_mid_i` is charged with multiplicity `k_i`,
/// is bounded above by `self.compose(d_mids)` under `adaptivity` adaptivity and `out`-composability.
/// Otherwise returns an error.
/// Composition is commutative, so this bound holds for any ordering of `d_mids`.
pub trait CompositionMeasure: Measure {
    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composability>;
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance>;
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_MaxDivergence.tex"
)]
impl CompositionMeasure for MaxDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        (d_mids.iter()).try_fold(0.0, |sum, (d_i, k_i)| {
            sum.inf_add(&d_i.inf_mul(&f64::from(*k_i))?)
        })
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ZeroConcentratedDivergence.tex"
)]
impl CompositionMeasure for ZeroConcentratedDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        (d_mids.iter()).try_fold(0.0, |sum, (d_i, k_i)| {
            sum.inf_add(&d_i.inf_mul(&f64::from(*k_i))?)
        })
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproximateMaxDivergence.tex"
)]
impl CompositionMeasure for Approximate<MaxDivergence> {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        (d_mids.iter()).try_fold((0.0, 0.0), |(eps_g, del_g), ((eps_i, del_i), k_i)| {
            let k_i = f64::from(*k_i);
            Ok((
                eps_g.inf_add(&eps_i.inf_mul(&k_i)?)?,
                del_g.inf_add(&del_i.inf_mul(&k_i)?)?,
            ))
        })
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_ApproximateZeroConcentratedDivergence.tex"
)]
impl CompositionMeasure for Approximate<ZeroConcentratedDivergence> {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Sequential)
    }
    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        (d_mids.iter()).try_fold((0.0, 0.0), |(eps_g, del_g), ((eps_i, del_i), k_i)| {
            let k_i = f64::from(*k_i);
            Ok((
                eps_g.inf_add(&eps_i.inf_mul(&k_i)?)?,
                del_g.inf_add(&del_i.inf_mul(&k_i)?)?,
            ))
        })
    }
}

#[proven(
    proof_path = "combinators/sequential_composition/CompositionMeasure_for_RenyiDivergence.tex"
)]
impl CompositionMeasure for RenyiDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composability> {
        Ok(Composability::Concurrent)
    }

    fn compose(&self, d_mids: Vec<(Self::Distance, u32)>) -> Fallible<Self::Distance> {
        // merge equal curves so that each is evaluated once, not once per copy
        let mut groups = OrderedCounts::new();
        for (d_mid, k_i) in d_mids {
            groups.add(d_mid, k_i)?;
        }
        Ok(Function::new_fallible(move |alpha| {
            (groups.iter())
                .map(|(curve, k)| curve.eval(alpha)?.inf_mul(&f64::from(*k)))
                .try_fold(0.0, |sum, eps| sum.inf_add(&eps?))
        }))
    }
}
