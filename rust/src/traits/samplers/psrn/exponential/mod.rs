use super::{InverseCDF, ODPRound};
use dashu::{float::FBig, rational::RBig};
use opendp_derive::proven;

#[cfg(test)]
mod test;

/// A partially sampled exponentially-distributed random number.
/// Initializes to span all reals.
///
/// A random variable follows the exponential distribution if it has density
///
/// ```math
/// f(x) = \frac{1}{\beta} e^{-\frac{x - \mu}{\beta}},
/// ```
/// where $\mu$ is the shift (location) parameter and $\beta$ is the scale parameter.
#[derive(Clone)]
pub struct ExponentialRV {
    /// finite
    pub shift: FBig,
    /// finite non-negative
    pub scale: FBig,
}

#[proven(proof_path = "traits/samplers/psrn/exponential/InverseCDF_for_ExponentialRV.tex")]
impl InverseCDF for ExponentialRV {
    type Edge = FBig;
    /// Retrieve either the lower or upper edge of the Exponential interval.
    /// Returns None if the sample is invalid- it must be refined more.
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
        let f_unif_comp = FBig::<R::C>::from(r_unif_comp)
            .with_precision(precision)
            .value();

        // infinity is not in the range
        if f_unif_comp == FBig::<R::C>::ZERO {
            return None;
        }
        let mut f_exp = (-f_unif_comp.ln()).with_rounding::<R>();

        f_exp *= self.scale.clone().with_rounding();
        f_exp += self.shift.clone().with_rounding();

        Some(f_exp.with_rounding())
    }
}
