use std::ops::AddAssign;

use rug::{float::Round, ops::NegAssign, Float, Integer, Rational};

use crate::error::Fallible;

use crate::traits::samplers::SampleStandardBernoulli;

use super::UniformPSRN;

/// A partially sampled tulap random number.
pub struct TulapPSRN {
    epsilon: Float,
    delta: Float,
    alpha: Float,
    uniform: UniformPSRN,
    precision: u32,
}

impl TulapPSRN {
    pub fn new(epsilon: Float, delta: Float, alpha: Float) -> Self {
        TulapPSRN {
            epsilon,
            delta,
            alpha,
            uniform: UniformPSRN::default(),
            precision: 1,
        }
    }

    pub fn value(&mut self, round: Round) -> Fallible<Rational> {
        loop {
            // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
            let uniform = Float::with_val_round(self.precision, self.uniform.value(round), round).0;

            if let Some(value) = Self::inverse_tulap(uniform) {
                return Ok(value);
            } else {
                self.refine()?;
            }
        }
    }
    fn q_cnd(u: Float, f: Float, c: Float) -> Float {
        if u < c {
            return TulapPSRN::q_cnd(1.0 - f * u, f, c) - 1.0;
        } else if u >= c && u <= 1.0 - c {
            return (u - 0.5) / (1.0 - 2.0 * c);
        } else {
            return TulapPSRN::q_cnd(f * (1.0 - u), f, c) + 1.0;
        }
    }

    fn inverse_tulap(epsilon: Float, delta: Float, alpha: Float) -> Float {
        // This is the spot you'd retrieve your uniform random number in Rust.
        // This would involve the UniformPSRN or another generator depending on your setup.
        let unif = Float::with_val_round(self.precision, self.uniform.value(round), round).0;

        let c = (1.0 - delta) / (1.0 + Float::exp(epsilon));
        let f = Float::max(
            0.0,
            1.0 - delta - Float::exp(epsilon) * alpha,
            Float::exp(-epsilon) * (1.0 - delta - alpha),
        );
        return TulapPSRN::q_cnd(unif, f, c);
    }

    pub fn refine(&mut self) -> Fallible<()> {
        self.precision += 1;
        self.uniform.refine()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let epsilon = Float::from(0.5);
        let delta = Float::from(0.25);
        let alpha = Float::from(0.1);

        let psrn = TulapPSRN::new(epsilon, delta, alpha);
        assert_eq!(psrn.epsilon, epsilon);
        assert_eq!(psrn.delta, delta);
        assert_eq!(psrn.alpha, alpha);
        assert_eq!(psrn.precision, 1);
    }

    #[test]
    fn test_value_calculation() {
        // Choose appropriate epsilon, delta, and alpha for your use case
        let epsilon = Float::from(0.5);
        let delta = Float::from(0.25);
        let alpha = Float::from(0.1);

        let mut psrn = TulapPSRN::new(epsilon, delta, alpha);

        // Ensure the value is within expected bounds.
        // This depends on your domain-specific expectations for Tulap.
        let value = psrn.value(Round::Up).unwrap();
        assert!(value >= 0.0 && value <= 1.0);
    }

    #[test]
    fn test_refining_behavior() {
        let epsilon = Float::from(0.5);
        let delta = Float::from(0.25);
        let alpha = Float::from(0.1);

        let mut psrn = TulapPSRN::new(epsilon, delta, alpha);
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
