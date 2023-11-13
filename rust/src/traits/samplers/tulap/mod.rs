
use rug::{float::Round, Float, Rational};

use crate::error::Fallible;

use super::UniformPSRN;

// TODO: make sure the implementation is still correct! Hope I didn't break anything
//       make sure the inverse tulap is done in a way where the rounding direction is always preserved!
//       fix tests
//       is there a clean way to do this with b and q instead of epsilon and delta? (epsilon = ln(1/b) and delta = q (1-b) / (2b(1-q)) )
//       if the user gives us the value of q and b, epsilon and delta can be calculated
//       pack into a constructor! (Mike)
//       remove tulap sampler from Python side
// 
//       update the proof to reflect this new implementation (if still correct)
//           - decide on b and q vs epsilon and delta
//               - I think scale and b are the same thing? But the pseudocode takes b and scale instead of scale and q?
//           - update pseudocode
//

/// A partially sampled tulap random number.
pub struct TulapPSRN {
    // b: Float, // b = exp(-eps), geom(p = 1 - b) - geom(p = 1 - b) ~ laplace(1 / eps)  ??
    // q: Float,
    shift: Rational,
    epsilon: Float,
    delta: Float,
    uniform: UniformPSRN,
    precision: u32,
}

impl TulapPSRN {
    // caliberate precision here to check if c is less than 0.5
    // sanity check the use of rounds in the inverse cdfs. 
    pub fn new(shift: Rational, epsilon: Float, delta: Float) -> Self {
        TulapPSRN {
            shift,
            epsilon,
            delta,
            uniform: UniformPSRN::default(),
            precision: 1, // check with a higer precision = 50 and if it runs. 
        }
    }

 //     
    pub fn value(&mut self, round: Round) -> Fallible<Rational> {
        loop {
            // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
            let uniform = Float::with_val_round(self.precision, self.uniform.value(round), round).0;
            println!("Generated uniform number: {}", uniform); 
            let tulap = self.inverse_tulap(uniform.clone(), round);
            println!("u: {}, tulap: {}", uniform, tulap); 
            if let Ok(value) = Rational::try_from(tulap) {
                return Ok(value + &self.shift);
            } else {
                self.refine()?;
            }
            
        }
    }

    // q cnd funtion explanation: 
    fn q_cnd(&self, u: Float, c: Float) -> Float {
        if u <  c.clone() {
            return self.q_cnd(1.0 - self.f(u), c.clone()) - 1.0;
        } else if u >=  c.clone() && u <= 1.0 - c.clone() {
            return (u - 0.5) / (1.0 - 2.0 * c.clone());
        } else {
            return self.q_cnd(self.f(1.0 - u),  c.clone()) + 1.0;
        }
    }

    fn inverse_tulap(&self, unif: Float, round: Round) -> Float {
        // more privacy makes c higher -> rounding up and this will make it more private
        // throw an error when c tends to 0.5
        // increment the precision until c is not 0.5 (line 34)
        let c = Float::with_val_round(self.precision, 1.0 - &self.delta, round).0 / (1.0 + Float::exp(self.epsilon.clone()));
        println!("The value of c is: {}", c);
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
mod test {
    use super::*;
    use rug::Float;

    #[test]
    fn test_sample_tulap_interval_progression() -> Fallible<()> {
        // change the value of epsilon and delta
        let epsilon = Float::with_val(52, 0.1);
        let delta = Float::with_val(52, 0.001);
        let mut tulap = TulapPSRN::new(Rational::from(0), epsilon, delta);

        for _ in 0..10 {
            println!(
                //"{:?}, {:?}",
                "{:?}, {:?}, {}",
                tulap.value(Round::Down)?.to_f64(),
                tulap.value(Round::Up)?.to_f64(),
                tulap.precision
            );
            tulap.refine()?;
        }
        Ok(())
    }

    #[test]
    fn test_tulap_psrn_samples() -> Fallible<()> {
        fn sample_tulap() -> Fallible<f64> {
            let epsilon = Float::with_val(52, 0.1);
            let delta = Float::with_val(52, 0.001);
            let mut tulap = TulapPSRN::new(Rational::from(0), epsilon, delta);

            for _ in 0..10 {
                tulap.refine()?;
            }
            Ok(tulap.value(Round::Down)?.to_f64())
        }

        let samples = (0..1000)
            .map(|_| sample_tulap())
            .collect::<Fallible<Vec<_>>>()?;

        println!("{:?}", samples);
        Ok(())
    }
}