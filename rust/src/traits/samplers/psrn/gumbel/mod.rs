use rug::{ops::NegAssign, Float, Rational};

use crate::error::Fallible;

use super::{Bound, UniformPSRN, PSRN};

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
            precision: 1,
        }
    }

    /// Computes the inverse cdf of the standard Gumbel with controlled rounding:
    /// $-ln(-ln(u))$ where $u \sim \mathrm{Uniform}(0, 1)$
    fn inverse_cdf(&self, mut sample: Float, bound: Bound) -> Option<Rational> {
        // This round is behind two negations, so the rounding direction is preserved
        sample.ln_round(bound.into());
        sample.neg_assign();

        // This round is behind a negation, so the rounding direction is reversed
        sample.ln_round(bound.complement().into());
        sample.neg_assign();

        Some(sample.to_rational()? + &self.shift)
    }
}

impl PSRN for GumbelPSRN {
    type Edge = Rational;
    /// Retrieve either the lower or upper edge of the Gumbel interval.
    /// The PSRN is refined until a valid value can be retrieved.
    fn edge(&mut self, bound: Bound) -> Fallible<Rational> {
        // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
        loop {
            let uniform = self.uniform.edge(bound)?;
            let uniform = Float::with_val_round(self.precision, uniform, bound.into()).0;
            if let Some(gumbel) = self.inverse_cdf(uniform, bound) {
                return Ok(gumbel);
            }
            self.refine()?;
        }
    }

    /// Improves the precision of the inverse transform,
    /// and halves the interval spanned by the uniform PSRN.
    fn refine(&mut self) -> Fallible<()> {
        self.precision += 1;
        self.uniform.refine()
    }

    fn refinements(&self) -> u32 {
        self.precision
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sample_gumbel_interval_progression() -> Fallible<()> {
        let mut gumbel = GumbelPSRN::new(Rational::from(0));
        for _ in 0..10 {
            println!(
                "{:?}, {:?}, {}",
                gumbel.edge(Bound::Lower)?.to_f64(),
                gumbel.edge(Bound::Upper)?.to_f64(),
                gumbel.precision
            );
            gumbel.refine()?;
        }
        Ok(())
    }

    #[test]
    fn test_gumbel_psrn() -> Fallible<()> {
        fn sample_gumbel() -> Fallible<f64> {
            let mut gumbel = GumbelPSRN::new(Rational::from(0));
            for _ in 0..10 {
                gumbel.refine()?;
            }
            Ok(gumbel.edge(Bound::Lower)?.to_f64())
        }
        let samples = (0..1000)
            .map(|_| sample_gumbel())
            .collect::<Fallible<Vec<_>>>()?;
        println!("{:?}", samples);
        Ok(())
    }
}
