use crate::core::{Function, Measurement, PrivacyMap, StabilityMap, Transformation};
use crate::error::Fallible;
use crate::measures::MaxDivergence;
use crate::metrics::SymmetricDistance;

use crate::domains::{AtomDomain, VectorDomain};
use crate::traits::CheckAtom;

pub fn make_test_measurement<T: 'static + Clone + CheckAtom>()
-> Fallible<Measurement<VectorDomain<AtomDomain<T>>, SymmetricDistance, MaxDivergence, T>> {
    Measurement::new(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance,
        MaxDivergence,
        Function::new(|arg: &Vec<T>| arg[0].clone()),
        PrivacyMap::new(|d_in| *d_in as f64 + 1.),
    )
}

pub fn make_test_transformation<T: Clone + CheckAtom>() -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        SymmetricDistance,
        VectorDomain<AtomDomain<T>>,
        SymmetricDistance,
    >,
> {
    Transformation::new(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance,
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance,
        Function::new(|arg: &Vec<T>| arg.clone()),
        StabilityMap::new_from_constant(1),
    )
}
