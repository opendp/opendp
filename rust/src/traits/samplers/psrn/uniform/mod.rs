use crate::{error::Fallible, traits::samplers::sample_standard_bernoulli};
use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
};

use super::{ODPRound, PSRN};

/// A partially sampled uniform random number.
/// Initializes to the interval [0, 1].
#[derive(Default)]
pub struct UniformPSRN {
    pub numer: UBig,
    /// The denominator is 2^denom_pow.
    pub denom_pow: usize,
}

impl PSRN for UniformPSRN {
    type Edge = RBig;
    // Retrieve either the lower or upper edge of the uniform interval.
    fn edge<R: ODPRound>(&self) -> Option<RBig> {
        Some(RBig::from_parts(
            IBig::from(self.numer.clone() + R::UBIG),
            UBig::ONE << self.denom_pow,
        ))
    }

    // Randomly discard the lower or upper half of the remaining interval.
    fn refine(&mut self) -> Fallible<()> {
        self.numer <<= 1;
        self.denom_pow += 1;
        if sample_standard_bernoulli()? {
            self.numer += UBig::ONE;
        }
        Ok(())
    }

    fn refinements(&self) -> usize {
        self.denom_pow
    }
}
