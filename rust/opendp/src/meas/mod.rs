//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use std::marker::PhantomData;

use num::NumCast;
use rand::Rng;

use crate::core::{Domain, Measure, Measurement, Metric, PrivacyRelation, Function};
use crate::dist::{L1Sensitivity, L2Sensitivity, MaxDivergence, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::{Fallible, Error};

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeMeasurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    fn make() -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make0()
    }
    fn make0() -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement1<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1> {
    fn make(param1: P1) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make1(param1)
    }
    fn make1(param1: P1) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement2<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2> {
    fn make(param1: P1, param2: P2) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make2(param1, param2)
    }
    fn make2(param1: P1, param2: P2) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement3<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3> {
    fn make(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make3(param1, param2, param3)
    }
    fn make3(param1: P1, param2: P2, param3: P3) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub trait MakeMeasurement4<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3, P4> {
    fn make(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>> {
        Self::make4(param1, param2, param3, param4)
    }
    fn make4(param1: P1, param2: P2, param3: P3, param4: P4) -> Fallible<crate::core::Measurement<DI, DO, MI, MO>>;
}

pub struct LaplaceMechanism<T> {
    data: PhantomData<T>
}

fn laplace(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5, 0.5);
    u.signum() * (1.0 - 2.0 * u.abs()).ln() * sigma
}

// laplace for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence, f64> for LaplaceMechanism<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence>> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                <f64 as NumCast>::from(*arg).and_then(|v| T::from(v + laplace(sigma))).ok_or_else(|| Error::FailedCast.into())
            }),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(1. / sigma)))
    }
}

pub struct VectorLaplaceMechanism<T> {
    data: PhantomData<T>
}

// laplace for vector-valued query
impl<T> MakeMeasurement1<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence, f64> for VectorLaplaceMechanism<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Fallible<Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence>> {
        Ok(Measurement::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            Function::new_fallible(move |arg: &Vec<T>| -> Fallible<Vec<T>> {
                arg.iter()
                    .map(|v| <f64 as NumCast>::from(*v).and_then(|v| T::from(v + laplace(sigma))))
                    .collect::<Option<_>>().ok_or_else(|| Error::FailedCast.into())
            }),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(1. / sigma)))
    }
}

pub struct GaussianMechanism<T> {
    data: PhantomData<T>
}

// gaussian for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence, f64> for GaussianMechanism<T>
    where T: Copy + NumCast {
    fn make1(sigma: f64) -> Fallible<Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence>> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new_fallible(move |arg: &T| -> Fallible<T> {
                <f64 as NumCast>::from(*arg).and_then(|v| T::from(v + laplace(sigma))).ok_or_else(|| Error::FailedCast.into())
            }),
            L2Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new_fallible(move |&d_in: &f64, &(eps, del): &(f64, f64)| {
                if d_in < 0. {
                    return Err(Error::InvalidDistance("gaussian mechanism: input sensitivity must be non-negative".to_string()).into())
                }
                if eps <= 0. {
                    return Err(Error::InvalidDistance("gaussian mechanism: epsilon must be positive".to_string()).into())
                }
                if del <= 0. {
                    return Err(Error::InvalidDistance("gaussian mechanism: delta must be positive".to_string()).into())
                }
                // TODO: should we error if epsilon > 1., or just waste the budget?
                Ok(eps.min(1.) >= (d_in / sigma) * (2. * (1.25 / del).ln()).sqrt())
            })))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_laplace_mechanism() {
        let measurement = LaplaceMechanism::<f64>::make(1.0).unwrap();
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.).unwrap());
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = VectorLaplaceMechanism::<f64>::make(1.0).unwrap();
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.).unwrap());
    }

    #[test]
    fn test_make_gaussian_mechanism() {
        let measurement = GaussianMechanism::<f64>::make(1.0).unwrap();
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)).unwrap());
    }

    #[test]
    fn test_error() {
        let measurement = GaussianMechanism::<f64>::make(1.0).unwrap();
        let error = measurement.privacy_relation.eval(&-0.1, &(0.5, 0.00001)).unwrap_err();
        let _backtrace = format!("{:?}", error.backtrace);
    }
}
