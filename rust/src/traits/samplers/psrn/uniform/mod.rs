use dashu::rational::RBig;

use crate::error::Fallible;

use super::{InverseCDF, ODPRound, PartialSample};

#[derive(Clone, Copy, Debug, Default)]
pub struct Uniform01RV;

impl InverseCDF for Uniform01RV {
    type Edge = RBig;

    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, _refinements: usize) -> Option<Self::Edge> {
        Some(uniform)
    }
}

#[cfg(test)]
mod test;

pub type PartialUniform01 = PartialSample<Uniform01RV>;

impl PartialUniform01 {
    /// Test `self < other / 2^shift` using aligned integer prefixes instead of
    /// constructing and reducing rational endpoints. Refine on overlap: no
    /// finite prefix is treated as a complete random real.
    pub(crate) fn less_than_scaled_uniform(
        &mut self,
        other: &mut Self,
        shift: usize,
    ) -> Fallible<bool> {
        loop {
            let other_precision = other.refinements + shift;
            let precision = self.refinements.max(other_precision);
            let self_shift = precision - self.refinements;
            let other_shift = precision - other_precision;
            let lo = &self.randomness << self_shift;
            let hi = (&self.randomness + 1u8) << self_shift;
            let other_lo = &other.randomness << other_shift;
            let other_hi = (&other.randomness + 1u8) << other_shift;

            if hi < other_lo {
                return Ok(true);
            }
            if other_hi < lo {
                return Ok(false);
            }
            if self.refinements <= other_precision {
                self.refine()?;
            } else {
                other.refine()?;
            }
        }
    }
}
