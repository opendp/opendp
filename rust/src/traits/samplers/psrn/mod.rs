use std::ops::AddAssign;

use rug::{float::Round, ops::NegAssign, Float, Integer, Rational};

use crate::{error::Fallible, traits::samplers::SampleStandardBernoulli};

/// A partially sampled uniform random number.
/// Initializes to the interval [0, 1].
#[derive(Default)]
pub struct UniformPSRN {
    pub numer: Integer,
    pub denom: u32,
}

impl UniformPSRN {
    // Retrieve either the lower or upper edge of the uniform interval.
    fn value(&self, round: Round) -> Rational {
        let round = match round {
            Round::Up => 1,
            Round::Down => 0,
            _ => panic!("value must be rounded Up or Down"),
        };
        Rational::from((self.numer.clone() + round, Integer::from(1) << self.denom))
    }
    // Randomly discard the lower or upper half of the remaining interval.
    fn refine(&mut self) -> Fallible<()> {
        self.numer <<= 1;
        self.denom += 1;
        if bool::sample_standard_bernoulli()? {
            self.numer += 1;
        }
        Ok(())
    }
}

/// A partially sampled Gumbel random number.
/// Initializes to span all reals.
pub struct GumbelPSRN {
    shift: Rational,
    uniform: UniformPSRN,
    precision: u32,
}

impl GumbelPSRN {
    pub fn new(shift: Rational) -> Self {
        GumbelPSRN {
            shift,
            uniform: UniformPSRN::default(),
            precision: 5,
        }
    }

    /// Retrieve either the lower or upper edge of the Gumbel interval.
    /// The PSRN is refined until a valid value can be retrieved.
    pub fn value(&mut self, round: Round) -> Fallible<Rational> {
        // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
        loop {
            let sample = Float::with_val(self.precision, self.uniform.value(round));

            if let Some(mut sample) = Self::inverse_cdf(sample, round).to_rational() {
                sample.add_assign(&self.shift);
                return Ok(sample);
            } else {
                self.refine()?;
            }
        }
    }

    /// Computes the inverse cdf of the standard Gumbel with controlled rounding:
    /// $-ln(-ln(u))$ where $u \sim \mathrm{Uniform}(0, 1)$
    fn inverse_cdf(mut sample: Float, round: Round) -> Float {
        fn complement(value: Round) -> Round {
            match value {
                Round::Up => Round::Down,
                Round::Down => Round::Up,
                _ => panic!("complement is only supported for Up/Down"),
            }
        }

        // This round is behind two negations, so the rounding direction is preserved
        sample.ln_round(round);
        sample.neg_assign();
        
        // This round is behind a negation, so the rounding direction is reversed
        sample.ln_round(complement(round));
        sample.neg_assign();

        sample
    }

    /// Improves the precision of the inverse transform,
    /// and halves the interval spanned by the uniform PSRN.
    pub fn refine(&mut self) -> Fallible<()> {
        self.precision += 5;
        self.uniform.refine()
    }

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals are disjoint.
    pub fn greater_than(&mut self, other: &mut Self) -> Fallible<bool> {
        Ok(loop {
            if self.value(Round::Down)? > other.value(Round::Up)? {
                break true;
            }
            if self.value(Round::Up)? < other.value(Round::Down)? {
                break false;
            }
            if self.precision < other.precision {
                self.refine()?
            } else {
                other.refine()?
            }
        })
    }
}
