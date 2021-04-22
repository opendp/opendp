use num::Float;

use crate::core::{Measurement, Function, PrivacyRelation};
use crate::dist::{L1Sensitivity, MaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::samplers::{SampleLaplace};
use crate::error::*;
use crate::traits::DistanceCast;

pub fn make_base_laplace<T>(scale: T) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<T>, MaxDivergence<T>>>
    where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &T| -> Fallible<T> {
            T::sample_laplace(*arg, scale, false)
        }),
        L1Sensitivity::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale.recip())
    ))
}

pub fn make_base_laplace_vec<T>(
    scale: T
) -> Fallible<Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<T>, MaxDivergence<T>>>

    where T: 'static + Clone + SampleLaplace + Float + DistanceCast {
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative")
    }
    Ok(Measurement::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<T>| -> Fallible<Vec<T>> {
            arg.iter()
                .map(|v| T::sample_laplace(*v, scale, false))
                .collect()
        }),
        L1Sensitivity::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_from_constant(scale.recip())
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_laplace_mechanism() {
        let measurement = make_base_laplace(1.0).unwrap_test();
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg).unwrap_test();

        assert!(measurement.privacy_relation.eval(&1., &1.).unwrap_test());
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = make_base_laplace_vec(1.0).unwrap_test();
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg).unwrap_test();

        assert!(measurement.privacy_relation.eval(&1., &1.).unwrap_test());
    }
}

