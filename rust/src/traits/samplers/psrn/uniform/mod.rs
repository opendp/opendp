use dashu::rational::RBig;

use super::{InverseCDF, ODPRound, PartialSample};

#[derive(Clone, Copy, Debug, Default)]
pub struct Uniform01;

impl InverseCDF for Uniform01 {
    type Edge = RBig;

    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, _refinements: usize) -> Option<Self::Edge> {
        Some(uniform)
    }
}

pub type PartialUniform01 = PartialSample<Uniform01>;
