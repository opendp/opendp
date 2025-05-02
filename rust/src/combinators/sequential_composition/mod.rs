#[cfg(feature = "contrib")]
mod non_adaptive;
#[cfg(feature = "contrib")]
pub use non_adaptive::*;

#[cfg(feature = "contrib")]
mod adaptive;
#[cfg(feature = "contrib")]
pub use adaptive::*;

#[cfg(feature = "ffi")]
mod ffi;

use crate::{
    core::{Function, Measure},
    error::Fallible,
    measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence},
    traits::InfAdd,
};

pub trait SequentialCompositionMeasure: Measure {
    fn concurrent(&self) -> Fallible<bool>;
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl SequentialCompositionMeasure for MaxDivergence {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(true)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

impl SequentialCompositionMeasure for ZeroConcentratedDivergence {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(true)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(0.0, |sum, d_i| sum.inf_add(d_i))
    }
}

impl SequentialCompositionMeasure for Approximate<MaxDivergence> {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(true)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        let (d_i0, deltas): (Vec<_>, Vec<_>) = d_i.into_iter().unzip();
        let delta = deltas
            .iter()
            .try_fold(0.0, |sum, delta| sum.inf_add(delta))?;

        Ok((self.0.compose(d_i0)?, delta))
    }
}

impl SequentialCompositionMeasure for Approximate<ZeroConcentratedDivergence> {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(false)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        let (d_i0, deltas): (Vec<_>, Vec<_>) = d_i.into_iter().unzip();
        let delta = deltas
            .iter()
            .try_fold(0.0, |sum, delta| sum.inf_add(delta))?;

        Ok((self.0.compose(d_i0)?, delta))
    }
}

impl SequentialCompositionMeasure for RenyiDivergence {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(true)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        Ok(Function::new_fallible(move |alpha| {
            d_i.iter()
                .map(|f| f.eval(alpha))
                .try_fold(0.0, |sum, e2| sum.inf_add(&e2?))
        }))
    }
}
