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
mod basic_composition;
#[cfg(feature = "contrib")]
pub use crate::combinators::basic_composition::*;

#[cfg(feature = "contrib")]
mod concurrent_composition;
#[cfg(feature = "contrib")]
pub use crate::combinators::concurrent_composition::*;

#[cfg(feature = "contrib")]
mod measure_cast;
#[cfg(feature = "contrib")]
pub use crate::combinators::measure_cast::*;

#[cfg(all(feature = "contrib", feature = "ffi"))]
mod user_defined;
#[cfg(all(feature = "contrib", feature = "ffi"))]
pub(crate) use crate::combinators::user_defined::*;

// #[cfg(feature = "contrib")]
// mod privacy_filter;
// #[cfg(feature = "contrib")]
// pub use crate::combinators::privacy_filter::*;

// #[cfg(feature = "contrib")]
// mod privacy_odometer;
// #[cfg(feature = "contrib")]
// pub use crate::combinators::privacy_odometer::*;

#[cfg(feature = "contrib")]
mod fix_delta;
#[cfg(feature = "contrib")]
pub use crate::combinators::fix_delta::*;

#[cfg(feature = "contrib")]
mod sequential_composition;
#[cfg(feature = "contrib")]
pub use crate::combinators::sequential_composition::*;

#[cfg(test)]
pub mod tests {
    use crate::core::{Function, PrivacyMap, StabilityMap, Transformation, Measurement1};
    use crate::measures::MaxDivergence;
    use crate::metrics::SymmetricDistance;
    use crate::domains::AllDomain;
    use crate::traits::CheckNull;

    pub fn make_test_measurement<T: 'static + Clone + CheckNull>(
    ) -> Measurement1<AllDomain<T>, AllDomain<T>, SymmetricDistance, MaxDivergence<f64>> {
        Measurement1::new1(
            AllDomain::new(),
            AllDomain::new(),
            Function::new(|arg: &T| arg.clone()),
            SymmetricDistance::default(),
            MaxDivergence::default(),
            PrivacyMap::new(|d_in| *d_in as f64 + 1.),
        )
    }

    pub fn make_test_transformation<T: Clone + CheckNull>() -> Transformation<AllDomain<T>, AllDomain<T>, SymmetricDistance, SymmetricDistance> {
        Transformation::new(
            AllDomain::default(),
            AllDomain::default(),
            Function::new(|arg: &T| arg.clone()),
            SymmetricDistance::default(),
            SymmetricDistance::default(),
            StabilityMap::new_from_constant(1)
        )
    }
}
