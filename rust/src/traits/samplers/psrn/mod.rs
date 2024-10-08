use std::fmt::Debug;

use dashu::{
    float::round::{
        mode::{Down, Up},
        Round,
    },
    integer::{IBig, UBig},
    rational::RBig,
};

use super::sample_from_uniform_bytes;
use crate::{error::Fallible, traits::RoundCast};

#[cfg(test)]
mod test;

mod gumbel;
pub use gumbel::GumbelRV;

pub trait InverseCDF: Sized {
    /// Type of lower or upper bound on the true random sample.
    type Edge: PartialOrd + Debug;

    /// Calculate either a lower or upper bound on the inverse cumulative distribution function.
    /// Returns None if the inverse CDF cannot be computed for the given uniform sample.
    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, refinements: usize) -> Option<Self::Edge>;
}

/// A partially sampled random number.
///
/// This representation of a random number is based on an arbitrarily precise sample from the uniform distribution,
/// as well as an inverse cumulative distribution function.
///
/// The exact value of the random number is unknown, but an interval around the sample is known.
/// This is because the uniform sample initializes to the interval [0, 1],
/// and each time the sample is refined, the interval for the uniform sample is narrowed.
pub struct PartialSample<D: InverseCDF> {
    // The numerator of the uniform sample fraction.
    randomness: UBig,
    /// The denominator of the uniform sample fraction is 2^refinements.
    refinements: usize,
    /// A struct from which you can compute the inverse CDF.
    pub distribution: D,
}

impl<D: InverseCDF> PartialSample<D> {
    pub fn new(distribution: D) -> Self {
        PartialSample {
            randomness: UBig::ZERO,
            refinements: 0,
            distribution,
        }
    }
}

impl<D: InverseCDF> PartialSample<D> {
    // Retrieve either the lower or upper edge of the uniform interval.
    fn edge<R: ODPRound>(&self) -> Option<D::Edge> {
        let uniform_edge = RBig::from_parts(
            IBig::from(self.randomness.clone() + R::UBIG),
            UBig::ONE << self.refinements,
        );

        self.distribution
            .inverse_cdf::<R>(uniform_edge, self.refinements)
    }

    /// Randomly discard the lower or upper half of the remaining interval 64 times.
    fn refine(&mut self) -> Fallible<()> {
        self.randomness <<= 64;
        self.randomness += UBig::from(sample_from_uniform_bytes::<u64, 8>()?);
        self.refinements += 64;
        Ok(())
    }

    /// Retrieve a lower bound for the true random sample.
    fn lower(&self) -> Option<D::Edge> {
        self.edge::<Down>()
    }

    /// Retrieve a upper bound for the true random sample.
    fn upper(&self) -> Option<D::Edge> {
        self.edge::<Up>()
    }

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals are disjoint.
    pub fn greater_than(
        self: &mut PartialSample<D>,
        other: &mut PartialSample<D>,
    ) -> Fallible<bool> {
        Ok(loop {
            match self.lower().zip(other.upper()) {
                Some((l, r)) if l > r => break true,
                _ => (),
            }
            match self.upper().zip(other.lower()) {
                Some((l, r)) if l < r => break false,
                _ => (),
            }

            if self.refinements < other.refinements {
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
                self.refine()?;
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
pub trait ODPRound: Round {
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
