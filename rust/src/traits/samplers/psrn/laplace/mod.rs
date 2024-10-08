use crate::error::Fallible;

use super::{InverseCDF, ODPRound};
use dashu::{float::FBig, integer::Sign, rational::RBig, rbig};

#[cfg(test)]
mod test;

/// A partially sampled Laplace random number.
/// Initializes to span all reals.
#[derive(Clone)]
pub struct LaplaceRV {
    shift: FBig,
    scale: FBig,
}

impl LaplaceRV {
    pub fn new(shift: FBig, scale: FBig) -> Fallible<Self> {
        if let Sign::Negative = scale.sign() {
            return fallible!(FailedFunction, "scale ({}) must be non-negative", scale);
        }
        Ok(LaplaceRV { shift, scale })
    }
}

impl InverseCDF for LaplaceRV {
    type Edge = FBig;

    /// Retrieve either the lower or upper edge of the Laplace interval.
    /// The PSRN is refined until a valid value can be retrieved.
    ///
    /// Computes the inverse cdf of the standard Laplace with controlled rounding:
    /// $+/- ln(u)$ where $u \sim \mathrm{Uniform}(0, 1)$
    ///
    /// When precision is low, return may be None due to the uniform PSRN initializing at zero.
    fn inverse_cdf<R: ODPRound>(&self, r_unif: RBig, refinements: usize) -> Option<FBig> {
        let precision = refinements + 1;
        let r_unif_double = r_unif * rbig!(2) - rbig!(1);

        // transform the uniform sample to a standard Laplace sample
        let mut sample_lap = match r_unif_double.sign() {
            // if heads, sample from [0, \infty)
            Sign::Positive => {
                // all operations prior to the negation should round in the opposite direction
                let r_unif_comp = RBig::ONE - r_unif_double;
                // infinity is not in the range
                if r_unif_comp == RBig::ZERO {
                    return None;
                }
                let f_uni = r_unif_comp.to_float::<R::C, 2>(precision).value();
                -f_uni.with_rounding::<R::C>().ln().with_rounding::<R>()
            }
            // if tails, sample from (-\infty, 0)
            Sign::Negative => {
                let r_unif = r_unif_double + RBig::ONE;

                // all operations round in the same direction
                // infinity is not in the range
                if r_unif == RBig::ZERO {
                    return None;
                }
                // don't double-sample zero
                if r_unif == RBig::ONE {
                    return None;
                }
                let f_uni = r_unif.to_float::<R, 2>(precision).value();
                f_uni.ln()
            }
        };

        sample_lap *= self.scale.clone().with_rounding::<R>();
        sample_lap += self.shift.clone().with_rounding::<R>();

        Some(sample_lap.with_rounding())
    }
}
