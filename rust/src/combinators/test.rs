use crate::core::{Function, Measurement, PrivacyMap, StabilityMap, Transformation};
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::SymmetricDistance;

use crate::domains::{AtomDomain, VectorDomain};
use crate::traits::CheckAtom;

pub fn make_test_measurement<T: 'static + Clone + CheckAtom>(
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence>> {
    Measurement::new(
        VectorDomain::new(AtomDomain::default()),
        Function::new(|arg: &Vec<T>| arg[0].clone()),
        SymmetricDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new(|d_in| *d_in as f64 + 1.),
    )
}

pub fn make_test_transformation<T: Clone + CheckAtom>() -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        VectorDomain<AtomDomain<T>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    Transformation::new(
        VectorDomain::new(AtomDomain::default()),
        VectorDomain::new(AtomDomain::default()),
        Function::new(|arg: &Vec<T>| arg.clone()),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    )
}
