use super::{InverseCDF, ODPRound, PSRN};
use dashu::{float::FBig, rational::RBig};

#[cfg(test)]
mod test;

/// A partially sampled Exponential random number.
/// Initializes to span all reals.
pub struct ExponentialDist {
    shift: FBig,
    scale: FBig,
}

impl ExponentialDist {
    pub fn new_psrn(shift: FBig, scale: FBig) -> PSRN<Self> {
        PSRN::new(ExponentialDist { shift, scale })
    }
}

impl InverseCDF for ExponentialDist {
    type Edge = FBig;
    /// Retrieve either the lower or upper edge of the Exponential interval.
    /// Returns None if the sample is invalid- it must be refined more
    ///
    /// First, computes the inverse cdf of the standard exponential with controlled rounding:
    /// $-ln(u)$ where $u \sim \mathrm{Uniform}(0, 1)$
    ///
    /// The return value is then shifted and scaled.
    ///
    /// When precision is low, return may be None due to the uniform PSRN initializing at zero.
    fn inverse_cdf<R: ODPRound>(&self, r_unif: RBig, refinements: usize) -> Option<FBig> {
        let precision = refinements + 1;
        let r_unif_comp = RBig::ONE - r_unif;
        let f_uni = FBig::<R>::from(r_unif_comp)
            .with_precision(precision)
            .value();

        // infinity is not in the range
        if f_uni == FBig::<R>::ZERO {
            return None;
        }
        let mut f_exp = -f_uni.with_rounding::<R::C>().ln();

        f_exp *= self.scale.clone().with_rounding();
        f_exp += self.shift.clone().with_rounding();

        Some(f_exp.with_rounding())
    }
}
