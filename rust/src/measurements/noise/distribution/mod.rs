use crate::{
    core::Domain,
    domains::{AtomDomain, VectorDomain},
    traits::CheckAtom,
};

mod gaussian;
pub use gaussian::*;

mod geometric;
pub use geometric::*;

mod laplace;
pub use laplace::*;

pub trait NoiseDomain: Domain {
    type Atom: 'static;
}

impl<T: 'static + CheckAtom> NoiseDomain for AtomDomain<T> {
    type Atom = T;
}

impl<T: 'static + CheckAtom> NoiseDomain for VectorDomain<AtomDomain<T>> {
    type Atom = T;
}
