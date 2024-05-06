use dashu::{
    base::Sign,
    float::{
        round::mode::{Down, Up},
        FBig,
    },
    rational::RBig,
};

use crate::error::Fallible;

use super::{ODPRound, UniformPSRN, PSRN};

#[cfg(test)]
mod test;

// TODO
//       update the proof to reflect this new implementation (if still correct)
//
// PYTHON
//     choose a final API for how to use the mechanism with postprocessing
//     examples: notebook to show how it works
//          for example: https://docs.opendp.org/en/stable/user/transformations/aggregation-quantile.html
//          ...comes from this notebook: https://github.com/opendp/opendp/blob/main/docs/source/user/transformations/aggregation-quantile.ipynb
//        a similar notebook in docs/source/user/measurements/tulap.ipynb demonstrating how/when to use it
//     code cleanup
//     documentation
//     another pass on naming
//     tests in python

/// A partially sampled tulap random number.
pub struct TulapPSRN {
    shift: RBig,
    exp_eps: RBig,
    neg_exp_eps: RBig,
    c: RBig,
    delta: RBig,
    uniform: UniformPSRN,
}

impl TulapPSRN {
    pub fn new(shift: RBig, epsilon: FBig, delta: RBig) -> Fallible<Self> {
        let exp_eps = epsilon.clone().with_rounding::<Down>().exp();
        let exp_eps = RBig::try_from(exp_eps)?;

        let neg_exp_eps = (-epsilon.clone()).with_rounding::<Up>().exp();
        let neg_exp_eps = RBig::try_from(neg_exp_eps)?;

        // c = (1 - δ) / (1 + exp(ε))
        let c = (RBig::ONE - &delta) / (RBig::ONE + &exp_eps);

        if c >= RBig::from_parts_const(Sign::Positive, 1, 2) {
            return fallible!(FailedFunction, "c must not exceed .5");
        }

        Ok(TulapPSRN {
            shift,
            exp_eps,
            neg_exp_eps,
            delta,
            c,
            uniform: UniformPSRN::default(),
        })
    }

    fn q_cnd(&self, unif: RBig) -> Option<RBig> {
        let _1 = RBig::ONE;

        Some(if unif < self.c {
            // in the setting where unif is less than c, like so:
            // --------|----|--------|--------|-------------
            //         u    c        .5       1-c
            self.q_cnd(&_1 - self.f(unif)?)? - _1
        } else if unif <= &_1 - &self.c {
            // in the setting where unif is within [c, 1 - c], like so:
            // -------------|----|---|--------|-------------
            //              c    u   .5       1-c
            let num = &unif - RBig::from_parts_const(Sign::Positive, 1, 2);
            let den = _1 - RBig::from(2) * &self.c;
            if den.is_zero() {
                return None;
            }
            num / den
        } else {
            // in the setting where unif is greater than 1 - c, like so:
            // -------------|--------|--------|----|--------
            //              c        .5       1-c  u
            self.q_cnd(self.f(&_1 - unif)?)? + _1
        })
    }

    fn f(&self, unif: RBig) -> Option<RBig> {
        let t1 = RBig::ONE - self.delta.clone() - &self.exp_eps * unif.clone();
        let t2 = &self.neg_exp_eps * (RBig::ONE - &self.delta - unif);
        Some(t1.max(t2).max(RBig::ZERO))
    }
}

impl PSRN for TulapPSRN {
    type Edge = RBig;
    fn edge<R: ODPRound>(&self) -> Option<RBig> {
        Some(self.q_cnd(self.uniform.edge::<R>()?)? + self.shift.clone())
    }

    fn refine(&mut self) -> Fallible<()> {
        self.uniform.refine()
    }

    fn refinements(&self) -> usize {
        self.uniform.refinements()
    }
}
