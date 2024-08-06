use std::fmt::Debug;

use dashu::{
    float::round::{
        mode::{Down, Up},
        ErrorBounds,
    },
    integer::{IBig, UBig},
    rational::RBig,
};

#[cfg(test)]
mod test;

mod gumbel;
pub use gumbel::GumbelDist;

use super::sample_standard_bernoulli;
use crate::{error::Fallible, traits::RoundCast};

mod tulap;
pub use tulap::TulapDist;

pub trait InverseCDF {
    /// Type of lower or upper bound on the true random sample.
    type Edge: PartialOrd + Debug;

    /// Calculate the inverse CDF
    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, refinements: usize) -> Option<Self::Edge>;
}

/// A partially sampled uniform random number.
///
/// This representation of a random number is based on an arbitrarily precise sample from the uniform distribution,
/// as well as an inverse cumulative distribution function.
///
/// The exact value of the random number is unknown, but an interval around the sample is known.
/// This is because the uniform sample initializes to the interval [0, 1],
/// and each time the sample is refined, the interval for the uniform sample is narrowed.
pub struct PSRN<D: InverseCDF> {
    // Numerator of the uniform sample fraction
    numer: UBig,
    /// The denominator is 2^denom_pow.
    denom_pow: usize,
    /// A struct from which you can compute the inverse CDF
    pub distribution: D,
}

impl<D: InverseCDF> PSRN<D> {
    fn new(distribution: D) -> Self {
        PSRN {
            numer: UBig::ZERO,
            denom_pow: 0,
            distribution,
        }
    }
}

impl<D: InverseCDF> PSRN<D> {
    // Retrieve either the lower or upper edge of the uniform interval.
    fn edge<R: ODPRound>(&self) -> Option<D::Edge> {
        let uniform_edge = RBig::from_parts(
            IBig::from(self.numer.clone() + R::UBIG),
            UBig::ONE << self.denom_pow,
        );

        self.distribution
            .inverse_cdf::<R>(uniform_edge, self.denom_pow)
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

    fn refinements(&self) -> usize {
        self.denom_pow
    }

    /// Retrieve the lower bound for the true random sample.
    fn lower(&self) -> Option<D::Edge> {
        self.edge::<Down>()
    }

    /// Retrieve the upper bound for the true random sample.
    fn upper(&self) -> Option<D::Edge> {
        self.edge::<Up>()
    }

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals are disjoint.
    pub fn is_gt(self: &mut PSRN<D>, other: &mut PSRN<D>) -> Fallible<bool> {
        Ok(loop {
            match self.lower().zip(other.upper()) {
                Some((l, r)) if l > r => break true,
                _ => (),
            }
            match self.upper().zip(other.lower()) {
                Some((l, r)) if l < r => break false,
                _ => (),
            }

            if self.refinements() < other.refinements() {
                self.refine()?
            } else {
                other.refine()?
            }
        })
    }

    /// Refine `psrn` until both bounds of interval round to same TO
    pub fn value<TO: RoundCast<D::Edge> + PartialEq>(&mut self) -> Fallible<TO> {
        Ok(loop {
            let Some((l, r)) = self.lower().zip(self.upper()) else {
                continue;
            };
            let (l, r) = (TO::round_cast(l)?, TO::round_cast(r)?);
            if l == r {
                break l;
            }
            self.refine()?;
        })
    }
}

/// Rounding directions used in PSRNs.
///
/// Implemented for Down and Up, respectively for lower and upper bounds.
pub trait ODPRound: ErrorBounds {
    /// * Down::UBIG = Zero
    /// * Up::UBIG = One
    const UBIG: UBig;

    /// Type of the complement.
    ///
    /// * Down::C = Up
    /// * Up::C = Down
    type C: ODPRound<C = Self>;
}

impl ODPRound for Down {
    const UBIG: UBig = UBig::ZERO;
    type C = Up;
}

impl ODPRound for Up {
    const UBIG: UBig = UBig::ONE;
    type C = Down;
}
