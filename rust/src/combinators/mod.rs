//! Various combinator constructors.

#[cfg(all(feature = "contrib", feature = "honest-but-curious"))]
mod amplify;
#[cfg(all(feature = "contrib", feature = "honest-but-curious"))]
pub use crate::combinators::amplify::*;

#[cfg(feature = "contrib")]
mod chain;
#[cfg(feature = "contrib")]
pub use crate::combinators::chain::*;

#[cfg(feature = "contrib")]
mod composition;
#[cfg(feature = "contrib")]
pub use crate::combinators::composition::*;

#[cfg(feature = "contrib")]
mod measure_cast;
#[cfg(feature = "contrib")]
pub use crate::combinators::measure_cast::*;

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
    use crate::core::{Function, Measurement, PrivacyMap, StabilityMap, Transformation};
    use crate::measures::MaxDivergence;
    use crate::metrics::SymmetricDistance;

    use crate::domains::{AtomDomain, VectorDomain};
    use crate::traits::CheckAtom;

    pub fn make_test_measurement<T: 'static + Clone + CheckAtom>(
    ) -> Measurement<VectorDomain<AtomDomain<T>>, T, SymmetricDistance, MaxDivergence<f64>> {
        Measurement::new(
            VectorDomain::new(AtomDomain::default(), None),
            Function::new(|arg: &Vec<T>| arg[0].clone()),
            SymmetricDistance::default(),
            MaxDivergence::default(),
            PrivacyMap::new(|d_in| *d_in as f64 + 1.),
        )
    }

    pub fn make_test_transformation<T: Clone + CheckAtom>() -> Transformation<
        VectorDomain<AtomDomain<T>>,
        VectorDomain<AtomDomain<T>>,
        SymmetricDistance,
        SymmetricDistance,
    > {
        Transformation::new(
            VectorDomain::new(AtomDomain::default(), None),
            VectorDomain::new(AtomDomain::default(), None),
            Function::new(|arg: &Vec<T>| arg.clone()),
            SymmetricDistance::default(),
            SymmetricDistance::default(),
            StabilityMap::new_from_constant(1),
        )
    }
}
