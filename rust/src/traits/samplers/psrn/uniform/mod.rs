use dashu::rational::RBig;

use super::{InverseCDF, ODPRound, PartialSample};

#[derive(Clone, Copy, Debug, Default)]
pub struct Uniform01RV;

impl InverseCDF for Uniform01RV {
    type Edge = RBig;

    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, _refinements: usize) -> Option<Self::Edge> {
        Some(uniform)
    }
}

pub type PartialUniform01 = PartialSample<Uniform01RV>;
