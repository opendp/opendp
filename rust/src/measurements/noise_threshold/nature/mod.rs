use crate::{
    core::Domain,
    domains::{AtomDomain, MapDomain},
    measurements::NoiseDomain,
    traits::{Hashable, Number},
};

mod float;
mod integer;

impl<DK: Domain, TV: Number> NoiseDomain for MapDomain<DK, AtomDomain<TV>>
where
    DK::Carrier: Hashable,
{
    type Atom = TV;
}
