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

#[cfg(feature = "ffi")]
mod ffi;

use crate::{
    core::{Function, Measure},
    error::Fallible,
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
    traits::InfAdd,
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
pub enum Composition {
    /// Previous interactive mechanisms are locked when a new query is submitted.
    Sequential,
    /// Previous interactive mechanisms are not locked when a new query is submitted.
    Concurrent,
}

pub trait CompositionMeasure: Measure {
    /// # Proof Definition
    /// For a given adaptivity and privacy measure,
    /// returns an error if composition is not valid,
    /// otherwise returns whether the privacy measure supports sequential or concurrent composition.
    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composition>;

    /// # Proof Definition
    /// For a given privacy measure, and list of privacy parameters `d_i`,
    /// returns the composition of the privacy parameters.
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl CompositionMeasure for MaxDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composition> {
        Ok(Composition::Concurrent)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

impl CompositionMeasure for ZeroConcentratedDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composition> {
        Ok(Composition::Concurrent)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

impl CompositionMeasure for Approximate<MaxDivergence> {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composition> {
        Ok(Composition::Concurrent)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        let (d_i0, deltas): (Vec<_>, Vec<_>) = d_i.into_iter().unzip();
        let delta = deltas
            .iter()
            .try_fold(0.0, |sum, delta| sum.inf_add(delta))?;

        Ok((self.0.compose(d_i0)?, delta))
    }
}

impl CompositionMeasure for Approximate<ZeroConcentratedDivergence> {
    fn composability(&self, adaptivity: Adaptivity) -> Fallible<Composition> {
        if matches!(adaptivity, Adaptivity::FullyAdaptive) {
            return fallible!(
                MakeMeasurement,
                "{adaptivity:?} composition is not supported for zCDP"
            );
        }
        Ok(Composition::Sequential)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        let (d_i0, deltas): (Vec<_>, Vec<_>) = d_i.into_iter().unzip();
        let delta = deltas
            .iter()
            .try_fold(0.0, |sum, delta| sum.inf_add(delta))?;

        Ok((self.0.compose(d_i0)?, delta))
    }
}

impl CompositionMeasure for RenyiDivergence {
    fn composability(&self, _adaptivity: Adaptivity) -> Fallible<Composition> {
        Ok(Composition::Concurrent)
    }

    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        Ok(Function::new_fallible(move |alpha| {
            d_i.iter()
                .map(|f| f.eval(alpha))
                .try_fold(0.0, |sum, e2| sum.inf_add(&e2?))
        }))
    }
}
