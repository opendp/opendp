use crate::error::Fallible;
use rug::float::Round;

mod gumbel;
pub use gumbel::GumbelPSRN;

mod laplace;
pub use laplace::LaplacePSRN;

mod uniform;
pub use uniform::UniformPSRN;

pub trait PSRN {
    type Edge: PartialOrd;
    fn edge(&mut self, bound: Bound) -> Fallible<Self::Edge>;
    fn refine(&mut self) -> Fallible<()>;
    fn refinements(&self) -> u32;

    /// Checks if `self` is greater than `other`,
    /// by refining the estimates for `self` and `other` until their intervals are disjoint.
    fn greater_than(&mut self, other: &mut Self) -> Fallible<bool> {
        Ok(loop {
            if self.edge(Lower)? > other.edge(Upper)? {
                break true;
            }
            if self.edge(Upper)? < other.edge(Lower)? {
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

// fn psrn_value<TI: PSRN<Edge = Rational>, TO: CastInternalRational + PartialEq>(
//     psrn: &mut TI,
// ) -> Fallible<TO> {
//     while TO::from_rational(psrn.edge(Lower)?) != TO::from_rational(psrn.edge(Upper)?) {
//         psrn.refine()?;
//     }
//     Ok(TO::from_rational(psrn.edge(Lower)?))
// }

#[derive(Copy, Clone)]
pub enum Bound {
    Lower = 0,
    Upper = 1,
}
use Bound::*;

impl Bound {
    fn complement(&self) -> Bound {
        match self {
            Lower => Upper,
            Upper => Lower,
        }
    }
}

impl From<Bound> for Round {
    fn from(value: Bound) -> Self {
        match value {
            Lower => Round::Down,
            Upper => Round::Up,
        }
    }
}
