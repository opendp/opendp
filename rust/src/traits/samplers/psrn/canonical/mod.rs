use crate::error::Fallible;

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
    pub tradeoff: &'a dyn Fn(RBig) -> Fallible<RBig>,
    pub fixed_point: &'a RBig,
}

#[proven(proof_path = "traits/samplers/psrn/canonical/InverseCDF_for_CanonicalRV.tex")]
impl<'a> InverseCDF for CanonicalRV<'a> {
    type Edge = RBig;
    fn inverse_cdf<R: ODPRound>(
        &self,
        uniform: RBig,
        _refinements: usize,
    ) -> Fallible<Option<RBig>> {
        let Some(v) = quantile_cnd(uniform, self.tradeoff, self.fixed_point)? else {
            return Ok(None);
        };

        Ok(Some(v * self.scale + &self.shift))
    }
}

/// # Proof Definition
/// Evaluates the quantile function $F^{-1}_f(u)$
/// as defined in Proposition F.6 of Canonical Noise Distributions and Private Hypothesis Tests,
/// where `uniform` is in $[0, 1]$, `f` is a symmetric nontrivial tradeoff function,
/// and `c` is the unique fixed point of `f`.
fn quantile_cnd(
    mut uniform: RBig,
    f: &dyn Fn(RBig) -> Fallible<RBig>,
    c: &RBig,
) -> Fallible<Option<RBig>> {
    // avoid infinite recursion / degenerate cases
    if uniform.is_zero() || uniform == rbig!(1) {
        return Ok(None);
    }

    let den = rbig!(1) - rbig!(2) * c;
    if den.is_zero() {
        return Ok(None);
    }

    let mut offset = rbig!(0);

    loop {
        if uniform.is_zero() || uniform == rbig!(1) {
            return Ok(None);
        }

        if &uniform < c {
            // in the setting where unif is less than c, like so:
            // --------|----|--------|--------|-------------
            //         u    c        .5       1-c
            let next = rbig!(1) - f(uniform.clone())?;

            // catches the exact GDP-mu=0 style fixed point
            if next == uniform {
                return Ok(None);
            }

            uniform = next;
            offset -= rbig!(1);
        } else if uniform <= rbig!(1) - c {
            // in the setting where unif is within [c, 1 - c], like so:
            // -------------|----|---|--------|-------------
            //              c    u   .5       1-c
            let num = uniform - rbig!(1 / 2);
            return Ok(Some(offset + num / den));
        } else {
            // in the setting where unif is greater than 1 - c, like so:
            // -------------|--------|--------|----|--------
            //              c        .5       1-c  u
            let next = f(rbig!(1) - uniform.clone())?;

            if next == uniform {
                return Ok(None);
            }

            uniform = next;
            offset += rbig!(1);
        }
    }
}
