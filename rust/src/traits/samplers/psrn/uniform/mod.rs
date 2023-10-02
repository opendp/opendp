use crate::{error::Fallible, traits::samplers::SampleStandardBernoulli};
use rug::{Integer, Rational};

use super::{PSRN, Bound};

/// A partially sampled uniform random number.
/// Initializes to the interval [0, 1].
#[derive(Default)]
pub struct UniformPSRN {
    pub numer: Integer,
    /// The denominator is 2^denom_pow.
    pub denom_pow: u32,
}

impl PSRN for UniformPSRN {
    type Edge = Rational;
    // Retrieve either the lower or upper edge of the uniform interval.
    fn edge(&mut self, bound: Bound) -> Fallible<Rational> {
        Ok(Rational::from((
            self.numer.clone() + bound as u8,
            Integer::from(1) << self.denom_pow,
        )))
    }
    // Randomly discard the lower or upper half of the remaining interval.
    fn refine(&mut self) -> Fallible<()> {
        self.numer <<= 1;
        self.denom_pow += 1;
        if bool::sample_standard_bernoulli()? {
            self.numer += 1;
        }
        Ok(())
    }

    fn refinements(&self) -> u32 {
        self.denom_pow
    }
}
