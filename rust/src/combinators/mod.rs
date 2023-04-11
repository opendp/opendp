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

#[cfg(all(feature = "contrib", feature = "ffi"))]
mod user_defined;

#[cfg(feature = "contrib")]
mod fix_delta;
#[cfg(feature = "contrib")]
pub use crate::combinators::fix_delta::*;

#[cfg(test)]
pub mod tests {
    use crate::core::{Function, Measurement, PrivacyMap, StabilityMap, Transformation};
    use crate::domains::AtomDomain;
    use crate::measures::MaxDivergence;
    use crate::metrics::SymmetricDistance;
    use crate::traits::CheckNull;

    pub fn make_test_measurement<T: Clone + CheckNull>(
    ) -> Measurement<AtomDomain<T>, T, SymmetricDistance, MaxDivergence<f64>> {
        Measurement::new(
            AtomDomain::new(),
            Function::new(|arg: &T| arg.clone()),
            SymmetricDistance::default(),
            MaxDivergence::default(),
            PrivacyMap::new(|d_in| *d_in as f64 + 1.),
        )
    }

    pub fn make_test_transformation<T: Clone + CheckNull>(
    ) -> Transformation<AtomDomain<T>, AtomDomain<T>, SymmetricDistance, SymmetricDistance> {
        Transformation::new(
            AtomDomain::default(),
            AtomDomain::default(),
            Function::new(|arg: &T| arg.clone()),
            SymmetricDistance::default(),
            SymmetricDistance::default(),
            StabilityMap::new_from_constant(1),
        )
    }
}
