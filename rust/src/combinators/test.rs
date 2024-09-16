use crate::core::{Function, Measurement, Odometer, PrivacyMap, StabilityMap, Transformation};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::measures::MaxDivergence;
use crate::metrics::{IntDistance, SymmetricDistance};

use crate::domains::{AtomDomain, VectorDomain};
use crate::traits::CheckAtom;

use super::{OdometerAnswer, OdometerCompositor, OdometerQuery};

pub fn make_test_odometer<T: 'static + Clone + CheckAtom>() -> Fallible<
    Odometer<
        VectorDomain<AtomDomain<T>>,
        OdometerCompositor<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence>,
        SymmetricDistance,
        MaxDivergence,
    >,
> {
    Odometer::new(
        VectorDomain::new(AtomDomain::default()),
        Function::new_interactive(|arg: &Vec<T>, wrapper| {
            let data = arg.clone();
            Queryable::new_external(
                move |query: &OdometerQuery<
                    Measurement<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence>,
                    IntDistance,
                >,
                      _| {
                    Ok(match query {
                        OdometerQuery::Invoke(meas) => OdometerAnswer::Invoke(meas.invoke(&data)?),
                        OdometerQuery::Map(d_in) => OdometerAnswer::Map(*d_in as f64 + 1.),
                    })
                },
                wrapper,
            )
        }),
        SymmetricDistance::default(),
        MaxDivergence::default(),
    )
}

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
