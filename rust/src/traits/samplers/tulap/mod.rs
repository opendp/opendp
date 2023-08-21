
use rug::{float::Round, Float, Rational};

use crate::error::Fallible;

use super::UniformPSRN;

// TODO: make sure the implementation is still correct! Hope I didn't break anything
//       make sure the inverse tulap is done in a way where the rounding direction is always preserved!
//       fix tests
//       pack into a constructor! (Mike)
//       remove tulap sampler from Python side
// 
//       update the proof to reflect this new implementation (if still correct)
//           - update pseudocode
//

/// A partially sampled tulap random number.
pub struct TulapPSRN {
    // b: Float, // b = exp(-eps), geom(p = 1 - b) - geom(p = 1 - b) ~ laplace(1 / eps)  ??
    // q: Float,
    epsilon: Float,
    delta: Float,
    uniform: UniformPSRN,
    precision: u32,
}

impl TulapPSRN {
    pub fn new(epsilon: Float, delta: Float) -> Self {
        TulapPSRN {
            epsilon,
            delta,
            uniform: UniformPSRN::default(),
            precision: 1,
        }
    }

    pub fn value(&mut self, round: Round) -> Fallible<Rational> {
        loop {
            // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
            let uniform = Float::with_val_round(self.precision, self.uniform.value(round), round).0;
            let tulap = self.inverse_tulap(uniform, round);

            if let Ok(value) = Rational::try_from(tulap) {
                return Ok(value);
            } else {
                self.refine()?;
            }
        }
    }

    fn q_cnd(&self, u: Float, c: Float) -> Float {
        if u < c {
            return self.q_cnd(1.0 - self.f(u), c.clone()) - 1.0;
        } else if u >= c && u <= 1.0 - c.clone() {
            return (u - 0.5) / (1.0 - 2.0 * c.clone());
        } else {
            return self.q_cnd(self.f(1.0 - u), c) + 1.0;
        }
    }

    fn inverse_tulap(&self, unif: Float, round: Round) -> Float {
        // This is the spot you'd retrieve your uniform random number in Rust.
        // This would involve the UniformPSRN or another generator depending on your setup.

        let c = Float::with_val_round(self.precision, 1.0 - &self.delta, round).0 / (1.0 + Float::exp(self.epsilon.clone()));
        return self.q_cnd(unif, c);
    }

    fn f(&self, alpha: Float) -> Float {
        let _1 = Float::with_val(52, 1.);
        // if this function can only be phrased in terms of ε, δ,
        // then we might as well keep everything in terms of ε, δ?

        let t1 = _1.clone() - &self.delta - Float::exp(self.epsilon.clone()) * alpha.clone();
        let t2 = Float::exp(-self.epsilon.clone()) * (_1 - &self.delta - alpha);
        t1.max(&t2).max(&Float::with_val(52, 0.))
    }

    pub fn refine(&mut self) -> Fallible<()> {
        self.precision += 1;
        self.uniform.refine()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_basic_creation() {
    //     let epsilon = Float::with_val(52, 0.5);
    //     let delta = Float::with_val(52, 0.25);

    //     let psrn = TulapPSRN::new(epsilon, delta);
    //     assert_eq!(psrn.epsilon, epsilon);
    //     assert_eq!(psrn.delta, delta);
    //     assert_eq!(psrn.precision, 1);
    // }

    #[test]
    fn test_value_calculation() {
        // Choose appropriate epsilon, delta, and alpha for your use case
        let epsilon = Float::with_val(52, 0.5);
        let delta = Float::with_val(52, 0.25);

        let mut psrn = TulapPSRN::new(epsilon, delta);

        // Ensure the value is within expected bounds.
        // This depends on your domain-specific expectations for Tulap.
        let value = psrn.value(Round::Up).unwrap();
        assert!(value >= 0.0 && value <= 1.0);
    }

    #[test]
    fn test_refining_behavior() {
        let epsilon = Float::with_val(52, 0.5);
        let delta = Float::with_val(52, 0.25);

        let mut psrn = TulapPSRN::new(epsilon, delta);
        assert_eq!(psrn.precision, 1);

        psrn.refine().unwrap();
        assert_eq!(psrn.precision, 2);
    }

    /*    #[test]
    fn test_inverse_tulap() {
        // This should be a direct test of known input-output pairs for inverse_tulap.
        // You would need to adjust the visibility of the inverse_tulap function to be public, or declare tests in the same module.
        let epsilon = Float::from(0.5);
        let delta = Float::from(0.25);
        let alpha = Float::from(0.1);

        let result = TulapPSRN::inverse_tulap(epsilon, delta, alpha);
        assert!(/* fill this up */);
    }  */
}
