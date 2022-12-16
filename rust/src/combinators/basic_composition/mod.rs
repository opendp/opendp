use num::Zero;

use crate::{
    core::Measure,
    error::Fallible,
    measures::{FixedSmoothedMaxDivergence, MaxDivergence, ZeroConcentratedDivergence},
    traits::InfAdd,
};

pub trait BasicCompositionMeasure: Measure {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for MaxDivergence<Q> {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for FixedSmoothedMaxDivergence<Q> {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter()
            .try_fold((Q::zero(), Q::zero()), |(e1, d1), (e2, d2)| {
                Ok((e1.inf_add(e2)?, d1.inf_add(d2)?))
            })
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for ZeroConcentratedDivergence<Q> {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}
