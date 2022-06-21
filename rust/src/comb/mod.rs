#[cfg(feature="contrib")]
pub mod amplify;
#[cfg(feature="contrib")]
pub use crate::comb::amplify::*;

#[cfg(feature="contrib")]
pub mod chain;
#[cfg(feature="contrib")]
pub use crate::comb::chain::*;

#[cfg(feature="contrib")]
pub mod compose;
#[cfg(feature="contrib")]
pub use crate::comb::compose::*;

#[cfg(feature="contrib")]
pub mod fix_delta;
#[cfg(feature="contrib")]
pub use crate::comb::fix_delta::*;



#[cfg(test)]
pub mod tests {
    use crate::core::{Function, Measurement, PrivacyMap, Transformation};
    use crate::core::{MaxDivergence, SymmetricDistance};
    use crate::core::AllDomain;
    use crate::error::*;
    use crate::traits::CheckNull;
    use crate::trans;

    pub fn make_test_measurement<T: Clone + CheckNull>() -> Measurement<AllDomain<T>, AllDomain<T>, SymmetricDistance, MaxDivergence<f64>> {
        Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new(|arg: &T| arg.clone()),
            SymmetricDistance::default(),
            MaxDivergence::default(),
            PrivacyMap::new(|_d_in| f64::INFINITY),
        )
    }

    pub fn make_test_transformation<T: Clone + CheckNull>() -> Transformation<AllDomain<T>, AllDomain<T>, SymmetricDistance, SymmetricDistance> {
        trans::make_identity(AllDomain::<T>::new(), SymmetricDistance::default()).unwrap_test()
    }
}
