use crate::error::Fallible;

use super::{InverseCDF, ODPRound};
use dashu::{base::Sign, float::FBig, rational::RBig};

#[cfg(test)]
mod test;

/// A Gumbel random variable.
#[derive(Clone)]
pub struct GumbelRV {
    shift: FBig,
    scale: FBig,
}

impl GumbelRV {
    pub fn new(shift: FBig, scale: FBig) -> Fallible<Self> {
        if let Sign::Negative = scale.sign() {
            return fallible!(FailedFunction, "scale ({}) must be non-negative", scale);
        }
        Ok(GumbelRV { shift, scale })
    }
}

impl InverseCDF for GumbelRV {
    type Edge = FBig;

    /// Computes the inverse cdf of the standard Gumbel with controlled rounding:
    /// $-ln(-ln(u))$ where $u \sim \mathrm{Uniform}(0, 1)$
    ///
    /// When precision is low, return may be None due to the uniform PSRN initializing at zero.
    fn inverse_cdf<R: ODPRound>(&self, r_uni: RBig, refinements: usize) -> Option<FBig> {
        let precision = refinements + 1;
        // These computations are behind two negations, so the rounding directions are preserved
        if r_uni == RBig::ZERO {
            return None;
        }
        let f_uni = r_uni.to_float::<R, 2>(precision).value();
        let f_exp = -f_uni.ln().with_precision(precision).value();

        if f_exp == FBig::<R>::ZERO {
            return None;
        }
        // These computations are behind one negation, so the rounding direction is reversed
        let f_exp = f_exp.with_rounding::<R::C>();
        let f_gumbel = -f_exp.ln().with_precision(precision).value();

        // Return to normal rounding for shift/scale
        let mut f_gumbel = f_gumbel.with_rounding::<R>();

        // Return to normal rounding for shift/scale
        f_gumbel *= self.scale.clone().with_rounding();
        f_gumbel += self.shift.clone().with_rounding();
        Some(f_gumbel.with_rounding())
    }
}
