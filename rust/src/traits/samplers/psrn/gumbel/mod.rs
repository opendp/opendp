use dashu::{float::FBig, rational::RBig};

use crate::error::Fallible;

use super::{ODPRound, UniformPSRN, PSRN};

/// A partially sampled Gumbel random number.
/// Initializes to span all reals.
pub struct GumbelPSRN {
    shift: RBig,
    scale: RBig,
    uniform: UniformPSRN,
    precision: usize,
}

impl GumbelPSRN {
    pub fn new(shift: RBig, scale: RBig) -> Self {
        GumbelPSRN {
            shift,
            scale,
            uniform: UniformPSRN::default(),
            precision: 20,
        }
    }
}

impl PSRN for GumbelPSRN {
    type Edge = RBig;

    /// Retrieve either the lower or upper edge of the Gumbel interval.
    /// Returns None if the sample is invalid- it must be refined more
    ///
    /// Computes the inverse cdf of the standard Gumbel with controlled rounding:
    /// $-ln(-ln(u))$ where $u \sim \mathrm{Uniform}(0, 1)$
    ///
    /// When precision is low, return may be None due to the uniform PSRN initializing at zero.
    fn edge<R: ODPRound>(&self) -> Option<RBig> {
        // These computations are behind two negations, so the rounding directions are preserved
        let r_uni = self.uniform.edge::<R>()?;
        if r_uni == RBig::ZERO {
            return None;
        }
        let f_uni = r_uni.to_float::<R, 2>(self.precision).value();
        let f_exp = -f_uni.ln().with_precision(self.precision).value();

        if f_exp == FBig::<R>::ZERO {
            return None;
        }
        // These computations are behind one negation, so the rounding direction is reversed
        let f_exp = f_exp.with_rounding::<R::C>();
        let f_gumbel = -f_exp.ln().with_precision(self.precision).value();

        // Return to normal rounding for shift/scale
        let f_gumbel = f_gumbel.with_rounding::<R>();
        let r_gumbel = RBig::simplest_from_float(&f_gumbel)?;
        Some(r_gumbel * &self.scale + &self.shift)
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
mod test;
