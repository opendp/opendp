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
    NonAdaptive,
    Adaptive,
    FullyAdaptive,
}

#[derive(Debug)]
pub enum Sequentiality {
    Sequential,
    Concurrent,
}

pub trait CompositionMeasure: Measure {
    fn theorem(&self, adaptivity: Adaptivity) -> Fallible<Sequentiality>;
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl CompositionMeasure for MaxDivergence {
    fn theorem(&self, _adaptivity: Adaptivity) -> Fallible<Sequentiality> {
        Ok(Sequentiality::Concurrent)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

impl CompositionMeasure for ZeroConcentratedDivergence {
    fn theorem(&self, _adaptivity: Adaptivity) -> Fallible<Sequentiality> {
        Ok(Sequentiality::Concurrent)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

impl CompositionMeasure for Approximate<MaxDivergence> {
    fn theorem(&self, _adaptivity: Adaptivity) -> Fallible<Sequentiality> {
        Ok(Sequentiality::Concurrent)
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
    fn theorem(&self, adaptivity: Adaptivity) -> Fallible<Sequentiality> {
        if matches!(adaptivity, Adaptivity::FullyAdaptive) {
            return fallible!(
                MakeMeasurement,
                "{adaptivity:?} composition is not supported for zCDP"
            );
        }
        Ok(Sequentiality::Sequential)
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
    fn theorem(&self, _adaptivity: Adaptivity) -> Fallible<Sequentiality> {
        Ok(Sequentiality::Concurrent)
    }

    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        Ok(Function::new_fallible(move |alpha| {
            d_i.iter()
                .map(|f| f.eval(alpha))
                .try_fold(0.0, |sum, e2| sum.inf_add(&e2?))
        }))
    }
}
