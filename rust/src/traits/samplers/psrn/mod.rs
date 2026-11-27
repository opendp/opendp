use std::fmt::Debug;

use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
};

use super::sample_from_uniform_bytes;
use crate::{
    error::Fallible,
    traits::{Direction, RoundCast},
};

#[cfg(all(feature = "contrib", test))]
mod test;

pub trait InverseCDF: Sized {
    /// Type of lower or upper bound on the true random sample.
    type Edge: PartialOrd + Debug;

    /// # Proof Definition
    /// Given a random variable `self` (of type `Self`) and `uniform` in [0, 1],
    /// return `Err(e)`, `Ok(None)` if a bound is unavailable, or `Ok(Some(out))`,
    /// where `out` bounds the inverse cumulative distribution function evaluated at `uniform`
    /// in the requested direction.
    ///
    /// As `refinements` tends to infinity and uniform inputs converge to an
    /// interior continuity point of the inverse CDF, bounds in both directions
    /// must eventually be available and converge to the exact quantile there.
    /// Successive enclosures need not be nested.
    fn inverse_cdf(
        &self,
        uniform: RBig,
        refinements: usize,
        direction: Direction,
    ) -> Fallible<Option<Self::Edge>>;
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
    /// Parameters about the distribution, like shift and scale, are stored in this struct.
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
    // Retrieve either a lower or upper bound on the sample.
    fn edge(&self, direction: Direction) -> Fallible<Option<D::Edge>> {
        let numerator = match direction {
            Direction::Down => self.randomness.clone(),
            Direction::Up => &self.randomness + UBig::ONE,
        };
        let uniform_edge = RBig::from_parts(IBig::from(numerator), UBig::ONE << self.refinements);

        self.distribution
            .inverse_cdf(uniform_edge, self.refinements, direction)
    }

    /// Randomly discard the lower or upper half of the remaining interval 64 times.
    fn refine(&mut self) -> Fallible<()> {
        self.randomness <<= 64;
        self.randomness += UBig::from(sample_from_uniform_bytes::<u64, 8>()?);
        self.refinements += 64;
        Ok(())
    }

    /// Retrieve a lower bound on the sample.
    fn lower(&self) -> Fallible<Option<D::Edge>> {
        self.edge(Direction::Down)
    }

    /// Retrieve an upper bound on the sample.
    fn upper(&self) -> Fallible<Option<D::Edge>> {
        self.edge(Direction::Up)
    }

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals establish the ordering.
    /// Terminates when the enclosures eventually certify the comparison.
    ///
    /// # Proof Definition
    /// Returns `true` if `self` is greater than `other`, and `false` otherwise.
    pub fn greater_than(
        self: &mut PartialSample<D>,
        other: &mut PartialSample<D>,
    ) -> Fallible<bool> {
        Ok(loop {
            match self.lower()?.zip(other.upper()?) {
                Some((l, r)) if l > r => break true,
                _ => (),
            }
            match self.upper()?.zip(other.lower()?) {
                Some((l, r)) if l <= r => break false,
                _ => (),
            }

            if self.refinements < other.refinements {
                self.refine()?
            } else {
                other.refine()?
            }
        })
    }

    /// Refine `psrn` until both bounds of interval round to same TO.
    /// Requires eventual agreement of the rounded bounds almost surely.
    ///
    /// # Proof Definition
    /// Returns a sample from the distribution with CDF as defined by `self`,
    /// rounded to the nearest value of type `TO`,
    /// or an error if there is a lack of system entropy.
    pub fn value<TO: RoundCast<D::Edge> + PartialEq>(&mut self) -> Fallible<TO> {
        Ok(loop {
            let Some((l, r)) = self.lower()?.zip(self.upper()?) else {
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
