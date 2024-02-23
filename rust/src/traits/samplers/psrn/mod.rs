use dashu::{
    float::round::{
        mode::{Down, Up},
        ErrorBounds,
    },
    integer::UBig,
};

mod gumbel;
pub use gumbel::GumbelPSRN;

mod laplace;
pub use laplace::LaplacePSRN;

mod uniform;
pub use uniform::UniformPSRN;

use crate::error::Fallible;

pub trait PSRN {
    type Edge: PartialOrd;
    fn edge<R: ODPRound>(&self) -> Option<Self::Edge>;
    fn refine(&mut self) -> Fallible<()>;
    fn refinements(&self) -> usize;

    fn lower(&self) -> Option<Self::Edge> {
        self.edge::<Down>()
    }
    fn upper(&self) -> Option<Self::Edge> {
        self.edge::<Up>()
    }

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals are disjoint.
    fn greater_than(&mut self, other: &mut Self) -> Fallible<bool> {
        Ok(loop {
            if self.lower() > other.upper() {
                break true;
            }
            if self.upper() < other.lower() {
                break false;
            }
            if self.refinements() < other.refinements() {
                self.refine()?
            } else {
                other.refine()?
            }
        })
    }
}

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

// fn psrn_value<TI: PSRN<Edge = Rational>, TO: CastInternalRational + PartialEq>(
//     psrn: &mut TI,
// ) -> Fallible<TO> {
//     while TO::from_rational(psrn.edge(Lower)?) != TO::from_rational(psrn.edge(Upper)?) {
//         psrn.refine()?;
//     }
//     Ok(TO::from_rational(psrn.edge(Lower)?))
// }

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn test_progression<RV: PSRN>(
        sampler: &mut RV,
        min_refinements: usize,
    ) -> (RV::Edge, RV::Edge)
    where
        RV::Edge: PartialOrd,
    {
        loop {
            sampler.refine().unwrap();
            let Some((l, r)) = sampler.lower().zip(sampler.upper()) else {
                continue;
            };
            assert!(l <= r);

            if sampler.refinements() >= min_refinements {
                return (l, r);
            }
        }
    }
}
