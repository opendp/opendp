use dashu::{
    float::{
        round::{
            mode::{Down, Up},
            ErrorBounds,
        },
        FBig,
    },
    integer::{IBig, UBig},
    rational::RBig,
};

use crate::error::Fallible;

use super::sample_standard_bernoulli;

pub trait ODPRound: ErrorBounds {
    const UBIG: UBig;
    type Complement: ODPRound<Complement = Self>;
}

impl ODPRound for Down {
    const UBIG: UBig = UBig::ZERO;
    type Complement = Up;
}

impl ODPRound for Up {
    const UBIG: UBig = UBig::ONE;
    type Complement = Down;
}

/// A partially sampled uniform random number.
/// Initializes to the interval [0, 1].
#[derive(Default)]
pub struct UniformPSRN {
    pub numer: UBig,
    /// The denominator is 2^denom_pow.
    pub denom_pow: usize,
}

impl UniformPSRN {
    // Retrieve either the lower or upper edge of the uniform interval.
    fn value<R: ODPRound>(&self) -> RBig {
        RBig::from_parts(
            IBig::from(self.numer.clone() + R::UBIG),
            UBig::ONE << self.denom_pow,
        )
    }
    // Randomly discard the lower or upper half of the remaining interval.
    fn refine(&mut self) -> Fallible<()> {
        self.numer <<= 1;
        self.denom_pow += 1;
        if sample_standard_bernoulli()? {
            self.numer += UBig::ONE;
        }
        Ok(())
    }
}

/// A partially sampled Gumbel random number.
/// Initializes to span all reals.
pub struct GumbelPSRN {
    shift: RBig,
    scale: RBig,
    uniform: UniformPSRN,
    precision: usize,
}

impl GumbelPSRN {
    pub fn new(shift: RBig, scale: RBig) -> Self {
        GumbelPSRN {
            shift,
            scale,
            uniform: UniformPSRN::default(),
            precision: 20,
        }
    }

    /// Retrieve either the lower or upper edge of the Gumbel interval.
    /// The PSRN is refined until a valid value can be retrieved.
    pub fn value<R: ODPRound>(&mut self) -> Fallible<RBig> {
        // The first few rounds are susceptible to NaN due to the uniform PSRN initializing at zero.
        loop {
            let r_uniform = self.uniform.value::<R>();
            if r_uniform.is_zero() {
                self.uniform.refine()?;
                continue;
            }
            let uniform = r_uniform.to_float::<R, 2>(self.precision).value();

            let Some(gumbel) = Self::inverse_cdf::<R>(uniform) else {
                self.refine()?;
                continue;
            };

            if let Some(gumbel) = RBig::simplest_from_float(&gumbel) {
                return Ok(gumbel * &self.scale + &self.shift);
            } else {
                self.refine()?;
            }
        }
    }

    /// Computes the inverse cdf of the standard Gumbel with controlled rounding:
    /// $-ln(-ln(u))$ where $u \sim \mathrm{Uniform}(0, 1)$
    fn inverse_cdf<R: ODPRound>(sample: FBig<R>) -> Option<FBig<R>> {
        let precision = sample.precision();
        // This round is behind two negations, so the rounding direction is preserved
        let sample = -sample.ln().with_precision(precision).value();

        if sample == FBig::<R>::ZERO {
            return None;
        }
        // This round is behind a negation, so the rounding direction is reversed
        let sample = sample.with_rounding::<R::Complement>();
        let sample = -sample.ln().with_precision(precision).value();

        Some(sample.with_rounding::<R>())
    }

    /// Improves the precision of the inverse transform,
    /// and halves the interval spanned by the uniform PSRN.
    pub fn refine(&mut self) -> Fallible<()> {
        self.precision += 1;
        self.uniform.refine()
    }

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals are disjoint.
    pub fn greater_than(&mut self, other: &mut Self) -> Fallible<bool> {
        Ok(loop {
            if self.value::<Down>()? > other.value::<Up>()? {
                break true;
            }
            if self.value::<Up>()? < other.value::<Down>()? {
                break false;
            }
            if self.precision < other.precision {
                self.refine()?
            } else {
                other.refine()?
            }
        })
    }
}

#[cfg(test)]
mod test;
