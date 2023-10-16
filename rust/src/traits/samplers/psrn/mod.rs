use num::traits::MulAddAssign;
use rug::{float::Round, ops::NegAssign, Float, Integer, Rational};

use crate::{error::Fallible, traits::samplers::SampleStandardBernoulli};

/// A partially sampled uniform random number.
/// Initializes to the interval [0, 1].
#[derive(Default)]
pub struct UniformPSRN {
    pub numer: Integer,
    /// The denominator is 2^denom_pow.
    pub denom_pow: u32,
}

impl UniformPSRN {
    // Retrieve either the lower or upper edge of the uniform interval.
    fn value(&self, round: Round) -> Rational {
        let round = match round {
            Round::Up => 1,
            Round::Down => 0,
            _ => panic!("value must be rounded Up or Down"),
        };
        Rational::from((self.numer.clone() + round, Integer::from(1) << self.denom_pow))
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
}

/// A partially sampled Gumbel random number.
/// Initializes to span all reals.
pub struct GumbelPSRN {
    shift: Rational,
    scale: Rational,
    uniform: UniformPSRN,
    precision: u32,
}

impl GumbelPSRN {
    pub fn new(shift: Rational, scale: Rational) -> Self {
        GumbelPSRN {
            shift,
            scale,
            uniform: UniformPSRN::default(),
            precision: 1,
        }
    }

    /// Retrieve either the lower or upper edge of the Gumbel interval.
    /// The PSRN is refined until a valid value can be retrieved.
    pub fn value(&mut self, round: Round) -> Fallible<Rational> {
        // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
        loop {
            let uniform = Float::with_val_round(self.precision, self.uniform.value(round), round).0;

            if let Some(mut gumbel) = Self::inverse_cdf(uniform, round).to_rational() {
                gumbel.mul_add_assign(&self.scale, &self.shift);
                return Ok(gumbel);
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
        self.precision += 1;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sample_gumbel_interval_progression() -> Fallible<()> {
        let mut gumbel = GumbelPSRN::new(Rational::from(0), Rational::from(1));
        for _ in 0..10 {
            println!(
                "{:?}, {:?}, {}",
                gumbel.value(Round::Down)?.to_f64(),
                gumbel.value(Round::Up)?.to_f64(),
                gumbel.precision
            );
            gumbel.refine()?;
        }
        Ok(())
    }

    #[test]
    fn test_gumbel_psrn() -> Fallible<()> {
        fn sample_gumbel() -> Fallible<f64> {
            let mut gumbel = GumbelPSRN::new(Rational::from(0), Rational::from(1));
            for _ in 0..10 {
                gumbel.refine()?;
            }
            Ok(gumbel.value(Round::Down)?.to_f64())
        }
        let samples = (0..1000)
            .map(|_| sample_gumbel())
            .collect::<Fallible<Vec<_>>>()?;
        println!("{:?}", samples);
        Ok(())
    }
}
