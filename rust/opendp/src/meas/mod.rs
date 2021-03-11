//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use std::marker::PhantomData;

use num::NumCast;
use rand::Rng;

use crate::core::{Domain, Measure, Measurement, Metric, PrivacyRelation};
use crate::dist::{L1Sensitivity, L2Sensitivity, MaxDivergence, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::Error;

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeMeasurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    fn make() -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error> {
        Self::make0()
    }
    fn make0() -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error>;
}

pub trait MakeMeasurement1<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1> {
    fn make(param1: P1) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error> {
        Self::make1(param1)
    }
    fn make1(param1: P1) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error>;
}

pub trait MakeMeasurement2<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2> {
    fn make(param1: P1, param2: P2) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error> {
        Self::make2(param1, param2)
    }
    fn make2(param1: P1, param2: P2) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error>;
}

pub trait MakeMeasurement3<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3> {
    fn make(param1: P1, param2: P2, param3: P3) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error> {
        Self::make3(param1, param2, param3)
    }
    fn make3(param1: P1, param2: P2, param3: P3) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error>;
}

pub trait MakeMeasurement4<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3, P4> {
    fn make(param1: P1, param2: P2, param3: P3, param4: P4) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error> {
        Self::make4(param1, param2, param3, param4)
    }
    fn make4(param1: P1, param2: P2, param3: P3, param4: P4) -> Result<crate::core::Measurement<DI, DO, MI, MO>, Error>;
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
impl MakeMeasurement1<AllDomain<f64>, AllDomain<f64>, L1Sensitivity<f64>, MaxDivergence, f64> for LaplaceMechanism<f64> {
    fn make1(sigma: f64) -> Result<Measurement<AllDomain<f64>, AllDomain<f64>, L1Sensitivity<f64>, MaxDivergence>, Error> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            move |v: &f64| *v + laplace(sigma),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(1. / sigma)
        ))
    }
}

pub struct VectorLaplaceMechanism<T> {
    data: PhantomData<T>
}

// laplace for vector-valued query
impl MakeMeasurement1<VectorDomain<AllDomain<f64>>, VectorDomain<AllDomain<f64>>, L1Sensitivity<f64>, MaxDivergence, f64> for VectorLaplaceMechanism<f64> {
    fn make1(sigma: f64) -> Result<Measurement<VectorDomain<AllDomain<f64>>, VectorDomain<AllDomain<f64>>, L1Sensitivity<f64>, MaxDivergence>, Error> {
        Ok(Measurement::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            move |arg: &Vec<f64>| arg.iter().map(|v| v + laplace(sigma)).collect(),
            L1Sensitivity::new(),
            MaxDivergence::new(),
            PrivacyRelation::new_from_constant(1. / sigma)
        ))
    }
}

pub struct GaussianMechanism<T> {
    data: PhantomData<T>
}

// gaussian for scalar-valued query
impl MakeMeasurement1<AllDomain<f64>, AllDomain<f64>, L2Sensitivity<f64>, SmoothedMaxDivergence, f64> for GaussianMechanism<f64>
    where f64: Copy + NumCast {
    fn make1(sigma: f64) -> Result<Measurement<AllDomain<f64>, AllDomain<f64>, L2Sensitivity<f64>, SmoothedMaxDivergence>, Error> {
        Ok(Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            // TODO: switch to gaussian
            enclose!(sigma, move |arg: &f64| arg + laplace(sigma)),
            L2Sensitivity::new(),
            SmoothedMaxDivergence::new(),
            PrivacyRelation::new(move |d_in: &f64, d_out: &(f64, f64)| {
                let (eps, delta) = d_out.clone();
                eps.min(1.) >= (*d_in / sigma) * (2. * (1.25 / delta).ln()).sqrt()
            })
        ))
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

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = VectorLaplaceMechanism::<f64>::make(1.0).unwrap();
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }

    #[test]
    fn test_make_gaussian_mechanism() {
        let measurement = GaussianMechanism::<f64>::make(1.0).unwrap();
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)));
    }
}
