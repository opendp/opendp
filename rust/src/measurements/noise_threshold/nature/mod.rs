use crate::{
    core::Domain,
    domains::{AtomDomain, MapDomain},
    measurements::NoiseDomain,
    traits::{CheckAtom, Hashable},
};

mod float;
mod integer;

impl<DK: Domain, TV: 'static + CheckAtom> NoiseDomain for MapDomain<DK, AtomDomain<TV>>
where
    DK::Carrier: Hashable,
{
    type Atom = TV;
}
