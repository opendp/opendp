use super::{ODPRound, UniformPSRN, PSRN};
use crate::error::Fallible;
use crate::traits::samplers::sample_standard_bernoulli;
use dashu::{float::FBig, integer::Sign, rational::RBig};

#[cfg(test)]
mod test;

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
            sign: Sign::from(sample_standard_bernoulli()?),
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
                let r_uni = self.uniform.edge::<R::C>()?;
                // infinity is not in the range
                if r_uni == RBig::ZERO {
                    return None;
                }
                let f_uni = r_uni.to_float::<R::C, 2>(self.precision).value();
                -f_uni.with_rounding::<R::C>().ln().with_rounding::<R>()
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
