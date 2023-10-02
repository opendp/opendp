use crate::traits::samplers::SampleStandardBernoulli;
use rug::ops::NegAssign;
use rug::Float;

use crate::error::Fallible;

use super::{Bound, UniformPSRN, PSRN};

/// A partially sampled Laplace random number.
/// Initializes to span all reals.
pub struct LaplacePSRN {
    shift: Float,
    scale: Float,
    uniform: UniformPSRN,
    precision: u32,
}

impl LaplacePSRN {
    pub fn new(shift: Float, scale: Float) -> Self {
        LaplacePSRN {
            shift,
            scale,
            uniform: UniformPSRN::default(),
            precision: 1,
        }
    }

    /// Computes the inverse cdf of the standard Laplace with controlled rounding:
    /// $+/- ln(u)$ where $u \sim \mathrm{Uniform}(0, 1)$
    fn inverse_cdf(&self, mut sample: Float, bound: Bound) -> Fallible<Float> {
        if bool::sample_standard_bernoulli()? {
            sample.ln_round(bound.into());
            sample.neg_assign();
        } else {
            sample.ln_round(bound.complement().into());
        }

        sample.mul_add_round(&self.scale, &self.shift, bound.into());
        Ok(sample)
    }
}

impl PSRN for LaplacePSRN {
    type Edge = Float;
    /// Retrieve either the lower or upper edge of the Laplace interval.
    /// The PSRN is refined until a valid value can be retrieved.
    fn edge(&mut self, bound: Bound) -> Fallible<Float> {
        // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
        loop {
            let uniform = self.uniform.edge(bound)?;
            let uniform = Float::with_val_round(self.precision, uniform, bound.into()).0;
            let laplace = self.inverse_cdf(uniform, bound)?;

            if !laplace.is_nan() {
                return Ok(laplace);
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
    fn test_sample_laplace_interval_progression() -> Fallible<()> {
        let mut laplace = LaplacePSRN::new(Float::with_val(53, 0.), Float::with_val(53, 1.));
        for _ in 0..10 {
            println!(
                "{:?}, {:?}, {}",
                laplace.edge(Bound::Lower)?.to_f64(),
                laplace.edge(Bound::Upper)?.to_f64(),
                laplace.precision
            );
            laplace.refine()?;
        }
        Ok(())
    }

    #[test]
    fn test_laplace_psrn() -> Fallible<()> {
        fn sample_laplace() -> Fallible<f64> {
            let mut laplace = LaplacePSRN::new(Float::with_val(53, 0.), Float::with_val(53, 1.));
            for _ in 0..10 {
                laplace.refine()?;
            }
            Ok(laplace.edge(Bound::Lower)?.to_f64())
        }
        let samples = (0..1000)
            .map(|_| sample_laplace())
            .collect::<Fallible<Vec<_>>>()?;
        println!("{:?}", samples);
        Ok(())
    }
}
