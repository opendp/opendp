use dashu::{
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
    rbig,
};
use opendp_derive::proven;

use crate::error::Fallible;

use super::{InverseCDF, ODPRound};

#[cfg(test)]
mod test;

/// An infinite-precision sample from the Tulap(shift, b, q) distribution,
/// where $b = \exp(-\epsilon)$ and $q = \frac{2\delta b}{1-b+2\delta b}$.
#[derive(Clone)]
pub struct TulapRV {
    shift: RBig,
    exp_eps: RBig,
    exp_neg_eps: RBig,
    c: RBig,
    delta: RBig,
}

impl TulapRV {
    pub fn new(shift: RBig, epsilon: FBig, delta: RBig) -> Fallible<Self> {
        // exp(ε)
        let exp_eps = epsilon.clone().with_rounding::<Down>().exp();
        let exp_eps = RBig::try_from(exp_eps)?;

        // exp(-ε)
        let exp_neg_eps = (-epsilon).with_rounding::<Up>().exp();
        let exp_neg_eps = RBig::try_from(exp_neg_eps)?;

        // c = (1 - δ) / (1 + exp(ε))
        let c = (rbig!(1) - &delta) / (rbig!(1) + &exp_eps);

        if c >= rbig!(1 / 2) {
            return fallible!(
                FailedFunction,
                "c must be less than 1/2. Please choose larger privacy parameters."
            );
        }

        Ok(TulapRV {
            shift,
            exp_eps,
            exp_neg_eps,
            delta,
            c,
        })
    }

    fn q_cnd(&self, unif: RBig) -> Option<RBig> {
        Some(if unif < self.c {
            // in the setting where unif is less than c, like so:
            // --------|----|--------|--------|-------------
            //         u    c        .5       1-c
            self.q_cnd(rbig!(1) - self.f(unif)?)? - rbig!(1)
        } else if unif <= rbig!(1) - &self.c {
            // in the setting where unif is within [c, 1 - c], like so:
            // -------------|----|---|--------|-------------
            //              c    u   .5       1-c
            let num = &unif - rbig!(1 / 2);
            let den = rbig!(1) - rbig!(2) * &self.c;
            if den.is_zero() {
                return None;
            }
            num / den
        } else {
            // in the setting where unif is greater than 1 - c, like so:
            // -------------|--------|--------|----|--------
            //              c        .5       1-c  u
            self.q_cnd(self.f(rbig!(1) - unif)?)? + rbig!(1)
        })
    }

    fn f(&self, unif: RBig) -> Option<RBig> {
        let t1 = rbig!(1) - &self.delta - &self.exp_eps * &unif;
        let t2 = &self.exp_neg_eps * (rbig!(1) - &self.delta - unif);
        Some(t1.max(t2).max(rbig!(0)))
    }
}

#[proven(proof_path = "traits/samplers/psrn/tulap/InverseCDF.tex")]
impl InverseCDF for TulapRV {
    type Edge = RBig;
    fn inverse_cdf<R: ODPRound>(&self, r_unif: RBig, _refinements: usize) -> Option<RBig> {
        Some(self.q_cnd(r_unif)? + &self.shift)
    }
}
