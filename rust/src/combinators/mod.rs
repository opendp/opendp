//! Various combinator constructors.

#[cfg(all(feature = "contrib", feature = "honest-but-curious"))]
mod amplify;
#[cfg(all(feature = "contrib", feature = "honest-but-curious"))]
pub use crate::combinators::amplify::*;

#[cfg(feature = "contrib")]
mod chain;
#[cfg(feature = "contrib")]
pub use crate::combinators::chain::*;

mod composition;
pub use crate::combinators::composition::*;

#[cfg(feature = "contrib")]
mod measure_cast;
#[cfg(feature = "contrib")]
pub use crate::combinators::measure_cast::*;

#[cfg(feature = "contrib")]
mod odometer;
#[cfg(feature = "contrib")]
pub use crate::combinators::odometer::*;

#[cfg(feature = "contrib")]
mod fix_delta;
#[cfg(feature = "contrib")]
pub use crate::combinators::fix_delta::*;

#[cfg(feature = "contrib")]
mod user_defined;
#[cfg(feature = "contrib")]
pub use crate::combinators::user_defined::*;

#[cfg(test)]
pub mod tests {
    use crate::core::{Function, Measurement, Odometer, PrivacyMap, StabilityMap, Transformation};
    use crate::error::Fallible;
    use crate::interactive::Queryable;
    use crate::measures::MaxDivergence;
    use crate::metrics::{IntDistance, SymmetricDistance};

    use crate::domains::{AtomDomain, VectorDomain};
    use crate::traits::CheckAtom;

    use super::{OdometerAnswer, OdometerQuery, OdometerQueryable};

    pub fn make_test_odometer<T: 'static + Clone + CheckAtom>() -> Fallible<
        Odometer<
            VectorDomain<AtomDomain<T>>,
            OdometerQueryable<usize, T, IntDistance, f64>,
            SymmetricDistance,
            MaxDivergence<f64>,
        >,
    > {
        Odometer::new(
            VectorDomain::new(AtomDomain::default()),
            Function::new_fallible(|arg: &Vec<T>| {
                let data = arg.clone();
                Queryable::new_external(move |query: &OdometerQuery<usize, IntDistance>| {
                    Ok(match query {
                        OdometerQuery::Invoke(idx) => OdometerAnswer::Invoke(data[*idx].clone()),
                        OdometerQuery::Map(d_in) => OdometerAnswer::Map(*d_in as f64 + 1.),
                    })
                })
            }),
            SymmetricDistance::default(),
            MaxDivergence::default(),
        )
    }

    pub fn make_test_measurement<T: 'static + Clone + CheckAtom>(
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence<f64>>>
    {
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
}
