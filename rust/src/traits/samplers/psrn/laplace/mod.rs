use super::{ODPRound, UniformPSRN, PSRN};
use crate::error::Fallible;
use crate::traits::samplers::SampleStandardBernoulli;
use dashu::{float::FBig, integer::Sign, rational::RBig};

/// A partially sampled Laplace random number.
/// Initializes to span all reals.
pub struct LaplacePSRN {
    shift: FBig,
    scale: FBig,
    uniform: UniformPSRN,
    precision: usize,
    sign: Sign,
}

impl LaplacePSRN {
    pub fn new(shift: FBig, scale: FBig) -> Fallible<Self> {
        Ok(LaplacePSRN {
            shift,
            scale,
            uniform: UniformPSRN::default(),
            precision: 1,
            sign: Sign::from(bool::sample_standard_bernoulli()?),
        })
    }
}

impl PSRN for LaplacePSRN {
    type Edge = FBig;

    /// Retrieve either the lower or upper edge of the Laplace interval.
    /// The PSRN is refined until a valid value can be retrieved.
    ///
    /// Computes the inverse cdf of the standard Laplace with controlled rounding:
    /// $+/- ln(u)$ where $u \sim \mathrm{Uniform}(0, 1)$
    ///
    /// When precision is low, return may be None due to the uniform PSRN initializing at zero.
    fn edge<R: ODPRound>(&self) -> Option<FBig> {
        // transform the uniform sample to a standard Laplace sample
        let mut sample_lap = match self.sign {
            // if heads, sample from [0, \infty)
            Sign::Positive => {
                // all operations prior to the negation should round in the opposite direction
                let r_uni = self.uniform.edge::<R::Complement>()?;
                // infinity is not in the range
                if r_uni == RBig::ZERO {
                    return None;
                }
                let f_uni = r_uni.to_float::<R::Complement, 2>(self.precision).value();
                -f_uni
                    .with_rounding::<R::Complement>()
                    .ln()
                    .with_rounding::<R>()
            }
            // if tails, sample from (-\infty, 0)
            Sign::Negative => {
                // all operations round in the same direction
                let r_uni = self.uniform.edge::<R>()?;
                // infinity is not in the range
                if r_uni == RBig::ZERO {
                    return None;
                }
                // don't double-sample zero
                if r_uni == RBig::ONE {
                    return None;
                }
                let f_uni = r_uni.to_float::<R, 2>(self.precision).value();
                f_uni.ln()
            }
        };

        sample_lap *= self.scale.clone().with_rounding::<R>();
        sample_lap += self.shift.clone().with_rounding::<R>();

        Some(sample_lap.with_rounding())
    }

    /// Improve the precision of the inverse transform,
    /// and halve the interval spanned by the uniform PSRN.
    fn refine(&mut self) -> Fallible<()> {
        self.precision += 1;
        self.uniform.refine()
    }

    fn refinements(&self) -> usize {
        self.precision
    }
}

#[cfg(test)]
mod test {
    use crate::traits::samplers::test::test_progression;

    use super::*;

    #[test]
    fn test_sample_laplace_interval_progression() -> Fallible<()> {
        let mut laplace = LaplacePSRN::new(FBig::ZERO, FBig::ONE)?;
        let (l, r) = test_progression(&mut laplace, 20);
        let (l, r) = (l.to_f64().value(), r.to_f64().value());
        println!("{l:?}, {r:?}, {}", laplace.refinements());
        Ok(())
    }

    #[test]
    fn test_laplace_psrn() -> Fallible<()> {
        fn sample_laplace() -> Fallible<f64> {
            let mut laplace = LaplacePSRN::new(FBig::ZERO, FBig::ONE)?;
            // refine it
            (0..30).try_for_each(|_| laplace.refine())?;

            Ok(laplace.lower().unwrap().to_f64().value())
        }
        let samples = (0..1000)
            .map(|_| sample_laplace())
            .collect::<Fallible<Vec<_>>>()?;
        println!("{:?}", samples);
        Ok(())
    }
}
