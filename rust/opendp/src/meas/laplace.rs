use std::marker::PhantomData;

use num::Float;

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L1Sensitivity, MaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::samplers::{SampleLaplace};
use crate::meas::{MakeMeasurement1};
use crate::error::Fallible;
use crate::traits::DistanceCast;

pub struct BaseLaplace<T> {
    data: PhantomData<T>
}

// laplace for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L1Sensitivity<T>, MaxDivergence<T>, T> for BaseLaplace<T>
    where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
    fn make1(scale: T) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<T>, MaxDivergence<T>>> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                T::sample_laplace(arg.clone(), scale.clone(), false)
            }),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(scale.recip())))
    }
}

pub struct BaseVectorLaplace<T> {
    data: PhantomData<T>
}

// laplace for vector-valued query
impl<T> MakeMeasurement1<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<T>, MaxDivergence<T>, T> for BaseVectorLaplace<T>
    where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
    fn make1(scale: T) -> Fallible<Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<T>, MaxDivergence<T>>> {
        Ok(Measurement::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            Function::new_fallible(move |arg: &Vec<T>| -> Fallible<Vec<T>> {
                arg.iter()
                    .map(|v| T::sample_laplace(v.clone(), scale, false))
                    .collect()
            }),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(scale.recip())))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_laplace_mechanism() {
        let measurement = BaseLaplace::<f64>::make(1.0).unwrap();
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg).unwrap();

        assert!(measurement.privacy_relation.eval(&1., &1.).unwrap());
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = BaseVectorLaplace::<f64>::make(1.0).unwrap();
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg).unwrap();

        assert!(measurement.privacy_relation.eval(&1., &1.).unwrap());
    }
}

