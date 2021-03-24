use std::marker::PhantomData;

use num::NumCast;

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L1Sensitivity, MaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::meas::{MakeMeasurement1, sample_laplace};
use crate::{Fallible, Error};

/// Univariate noise addition for the Laplace Mechanism
/// [Accompanying Proof](https://www.overleaf.com/read/brvrprjhrhwb)
pub struct BaseLaplace<T> {
    data: PhantomData<T>
}

// laplace for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence, f64> for BaseLaplace<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence>> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                <f64 as NumCast>::from(*arg).ok_or(Error::FailedCast)
                    .and_then(|v| sample_laplace(sigma).and_then(|noise| T::from(v + noise).ok_or(Error::FailedCast)))
            }),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(1. / sigma)))
    }
}

/// Multivariate noise addition for the Laplace Mechanism.
/// [Accompanying Proof](https://www.overleaf.com/read/brvrprjhrhwb)
pub struct BaseVectorLaplace<T> {
    data: PhantomData<T>
}

// laplace for vector-valued query
impl<T> MakeMeasurement1<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence, f64> for BaseVectorLaplace<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Fallible<Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence>> {
        Ok(Measurement::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            Function::new_fallible(move |arg: &Vec<T>| -> Fallible<Vec<T>> {
                arg.iter()
                    .map(|v| <f64 as NumCast>::from(*v).ok_or(Error::FailedCast)
                        .and_then(|v| sample_laplace(sigma)
                            .and_then(|noise| T::from(v + noise).ok_or(Error::FailedCast))))
                    .collect::<Fallible<_>>()
            }),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(1. / sigma)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_laplace_mechanism() {
        let measurement = BaseLaplace::<f64>::make(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = BaseVectorLaplace::<f64>::make(1.0);
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }
}

