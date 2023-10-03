use super::{ODPRound, UniformPSRN, PSRN};
use crate::error::Fallible;
use dashu::{float::FBig, rational::RBig};

/// A partially sampled Exponential random number.
/// Initializes to span all reals.
pub struct ExponentialPSRN {
    shift: RBig,
    scale: RBig,
    uniform: UniformPSRN,
    precision: usize,
}

impl ExponentialPSRN {
    pub fn new(shift: RBig, scale: RBig) -> Self {
        ExponentialPSRN {
            shift,
            scale,
            uniform: UniformPSRN::default(),
            precision: 1,
        }
    }
}

impl PSRN for ExponentialPSRN {
    type Edge = RBig;
    /// Retrieve either the lower or upper edge of the Exponential interval.
    /// The PSRN is refined until a valid value can be retrieved.
    fn edge<R: ODPRound>(&self) -> Option<RBig> {
        let f_uni = FBig::<R>::from(self.uniform.edge::<R::Complement>()?)
            .with_precision(self.precision)
            .value();

        // infinity is not in the range
        if f_uni == FBig::<R>::ZERO {
            return None;
        }
        let f_exp = -f_uni.with_rounding::<R::Complement>().ln();

        let Some(mut r_exp) = RBig::simplest_from_float(&f_exp) else {
            return None;
        };

        r_exp *= &self.scale;
        r_exp += &self.shift;

        Some(r_exp)
    }

    /// Improves the precision of the inverse transform,
    /// and halves the interval spanned by the uniform PSRN.
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
    use crate::traits::samplers::psrn::test::test_progression;

    use super::*;

    #[test]
    fn test_sample_exponential_interval_progression() -> Fallible<()> {
        let mut exp = ExponentialPSRN::new(RBig::ZERO, RBig::ONE);
        let (l, r) = test_progression(&mut exp, 20);
        let (l, r) = (l.to_f64().value(), r.to_f64().value());
        println!("{l:?}, {r:?}, {}", exp.refinements());
        Ok(())
    }

    #[test]
    fn test_exponential_psrn() -> Fallible<()> {
        fn sample_exponential() -> Fallible<f64> {
            let mut exp = ExponentialPSRN::new(RBig::ZERO, RBig::ONE);
            // refine it
            (0..30).try_for_each(|_| exp.refine())?;

            Ok(exp.lower().unwrap().to_f64().value())
        }
        let samples = (0..1000)
            .map(|_| sample_exponential())
            .collect::<Fallible<Vec<_>>>()?;
        println!("{:?}", samples);
        Ok(())
    }
}
