use dashu::float::FBig;

use crate::error::Fallible;

use super::{ODPRound, UniformPSRN, PSRN};

// TODO
//
//
// RUST
//     make sure the inverse tulap is done in a way where the rounding direction is always preserved!
//     check that the partial psrn is not making the sampling worse
//
// PROOF
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
    shift: FBig,
    epsilon: FBig,
    delta: FBig,
    uniform: UniformPSRN,
    precision: usize,
}

impl TulapPSRN {
    // caliberate precision here to check if c is less than 0.5
    // sanity check the use of rounds in the inverse cdfs.
    pub fn new(shift: FBig, epsilon: FBig, delta: FBig) -> Self {
        TulapPSRN {
            shift,
            epsilon,
            delta,
            uniform: UniformPSRN::default(),
            precision: 50,
        }
    }

    // q cnd funtion explanation:
    fn q_cnd<R: ODPRound>(&self, u: FBig<R>, c: FBig<R>) -> FBig<R> {
        let _1 = FBig::<R>::ONE.with_precision(self.precision).value();
        if u < c.clone() {
            self.q_cnd(_1 - self.f(u), c.clone()) - FBig::ONE
        } else if u >= c.clone() && u <= &_1 - c.clone() {
            (u - &_1 / FBig::from(2)) / (_1 - FBig::from(2) * c.clone())
        } else {
            self.q_cnd(self.f(&_1 - u), c.clone()) + _1
        }
    }

    fn f<R: ODPRound>(&self, u: FBig<R>) -> FBig<R> {
        // if this function can only be phrased in terms of ε, δ,
        // then we might as well keep everything in terms of ε, δ?
        let _1 = FBig::<R>::ONE.with_precision(self.precision).value();

        let t1 = &_1
            - self.delta.clone().with_rounding()
            - self.epsilon.clone().with_rounding().exp() * u.clone();
        let t2 = (-self.epsilon.clone().with_rounding()).exp()
            * (_1 - &self.delta.clone().with_rounding() - u);
        t1.max(t2).max(FBig::<R>::ZERO)
    }
}

impl PSRN for TulapPSRN {
    type Edge = FBig;
    fn edge<R: ODPRound>(&self) -> Option<FBig> {
        let uniform = FBig::<R>::from(self.uniform.edge::<R>()?)
            .with_precision(self.precision)
            .value();
        let _1 = FBig::<R>::ONE.with_precision(self.precision).value();
        let c = (&_1 - self.delta.clone().with_rounding())
            / (&_1 + self.epsilon.clone().with_rounding().exp());

        if c == FBig::<R>::try_from(0.5).unwrap() {
            return None;
        }

        let tulap = self.q_cnd(uniform, c) + self.shift.clone().with_rounding();
        Some(tulap.with_rounding())
    }

    fn refine(&mut self) -> Fallible<()> {
        self.precision += 1;
        self.uniform.refine()
    }

    fn refinements(&self) -> usize {
        self.precision
    }
}

#[cfg(test)]
mod test {
    use crate::traits::samplers::test::test_progression;

    use super::*;

    #[test]
    fn test_sample_tulap_interval_progression() -> Fallible<()> {
        let mut tulap = TulapPSRN::new(
            FBig::ZERO,
            FBig::ONE.with_precision(50).value(),
            FBig::try_from(1e-6).unwrap(),
        );
        let (l, r) = test_progression(&mut tulap, 20);
        let (l, r) = (l.to_f64().value(), r.to_f64().value());
        println!("{l:?}, {r:?}, {}", tulap.refinements());
        Ok(())
    }

    #[test]
    fn test_tulap_psrn() -> Fallible<()> {
        fn sample_tulap() -> Fallible<f64> {
            let mut tulap = TulapPSRN::new(
                FBig::ZERO,
                FBig::ONE.with_precision(50).value(),
                FBig::try_from(1e-6).unwrap(),
            );
            // refine it
            (0..30).try_for_each(|_| tulap.refine())?;

            Ok(tulap.lower().unwrap().to_f64().value())
        }
        let samples = (0..1000)
            .map(|_| sample_tulap())
            .collect::<Fallible<Vec<_>>>()?;
        println!("{:?}", samples);
        Ok(())
    }
}
