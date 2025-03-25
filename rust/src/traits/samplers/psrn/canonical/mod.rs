use super::{InverseCDF, ODPRound};
use dashu::{rational::RBig, rbig};
use opendp_derive::proven;

#[cfg(all(feature = "contrib", test))]
mod test;

/// A random variable representing a shifted and scaled canonical noise distribution.
///
/// The inverse CDF is defined to be F^{-1}_f(u) * shift + scale,
/// where f is the tradeoff function.
#[derive(Clone)]
pub struct CanonicalRV<'a> {
    pub shift: RBig,
    pub scale: &'a RBig,
    pub tradeoff: &'a dyn Fn(RBig) -> RBig,
    pub fixed_point: &'a RBig,
}

#[proven(proof_path = "traits/samplers/psrn/canonical/InverseCDF_for_CanonicalRV.tex")]
impl<'a> InverseCDF for CanonicalRV<'a> {
    type Edge = RBig;
    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, _refinements: usize) -> Option<RBig> {
        Some(quantile_cnd(uniform, self.tradeoff, self.fixed_point)? * self.scale + &self.shift)
    }
}

/// # Proof Definition
/// Evaluates the quantile function $F^{-1}_f(u)$
/// as defined in Proposition F.6 of Canonical Noise Distributions and Private Hypothesis Tests,
/// where `uniform` is in $[0, 1]$, `f` is a symmetric nontrivial tradeoff function,
/// and `c` is the unique fixed point of `f`.
fn quantile_cnd(uniform: RBig, f: &dyn Fn(RBig) -> RBig, c: &RBig) -> Option<RBig> {
    Some(if &uniform < c {
        // in the setting where unif is less than c, like so:
        // --------|----|--------|--------|-------------
        //         u    c        .5       1-c
        quantile_cnd(rbig!(1) - f(uniform), f, c)? - rbig!(1)
    } else if uniform <= rbig!(1) - c {
        // in the setting where unif is within [c, 1 - c], like so:
        // -------------|----|---|--------|-------------
        //              c    u   .5       1-c
        let num = &uniform - rbig!(1 / 2);
        let den = rbig!(1) - rbig!(2) * c;
        if den.is_zero() {
            return None;
        }
        num / den
    } else {
        // in the setting where unif is greater than 1 - c, like so:
        // -------------|--------|--------|----|--------
        //              c        .5       1-c  u
        quantile_cnd(f(rbig!(1) - uniform), f, c)? + rbig!(1)
    })
}
