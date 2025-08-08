use crate::core::{
    Function, Measurement, Odometer, OdometerAnswer, OdometerQuery, PrivacyMap, StabilityMap,
    Transformation,
};
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::measures::MaxDivergence;
use crate::metrics::SymmetricDistance;

use crate::domains::{AtomDomain, VectorDomain};
use crate::traits::CheckAtom;

pub fn make_test_odometer<T: 'static + Clone + CheckAtom>() -> Fallible<
    Odometer<
        VectorDomain<AtomDomain<T>>,
        SymmetricDistance,
        MaxDivergence,
        Measurement<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence>,
        T,
    >,
> {
    Odometer::new(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
        MaxDivergence::default(),
        Function::new_fallible(|arg: &Vec<T>| {
            let data = arg.clone();
            Queryable::new_external(
                move |query: &OdometerQuery<
                    Measurement<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence>,
                    u32,
                >| {
                    Ok(match query {
                        OdometerQuery::Invoke(meas) => OdometerAnswer::Invoke(meas.invoke(&data)?),
                        OdometerQuery::PrivacyLoss(_) => OdometerAnswer::PrivacyLoss(1.),
                    })
                },
            )
        }),
    )
}

pub fn make_test_measurement<T: 'static + Clone + CheckAtom>()
-> Fallible<Measurement<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence>> {
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
        VectorDomain<AtomDomain<T>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    Transformation::new(
        VectorDomain::new(AtomDomain::default()),
        VectorDomain::new(AtomDomain::default()),
        Function::new(|arg: &Vec<T>| arg.clone()),
        SymmetricDistance,
        SymmetricDistance,
        StabilityMap::new_from_constant(1),
    )
}
