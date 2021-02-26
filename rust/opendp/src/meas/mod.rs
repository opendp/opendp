//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use rand::Rng;

use crate::core::{Measurement, Domain, Measure, Metric};
use crate::dist::{L2Sensitivity, L1Sensitivity, MaxDivergence, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use num::NumCast;
use std::marker::PhantomData;

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeMeasurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    fn make() -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement1<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1> {
    fn make(param1: P1) -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement2<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2> {
    fn make(param1: P1, param2: P2) -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement3<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3> {
    fn make(param1: P1, param2: P2, param3: P3) -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement4<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3, P4> {
    fn make(param1: P1, param2: P2, param3: P3, param4: P4) -> crate::core::Measurement<DI, DO, MI, MO>;
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
    fn make(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence> {
        let input_domain = AllDomain::new();
        let output_domain = AllDomain::new();
        let function = move |arg: &T| -> T {
            <f64 as NumCast>::from(*arg).and_then(|v| T::from(v + laplace(sigma))).unwrap()
        };
        let input_metric = L1Sensitivity::new();
        let output_measure = MaxDivergence::new();
        let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
        Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}

pub struct VectorLaplaceMechanism<T> {
    data: PhantomData<T>
}

// laplace for vector-valued query
impl<T> MakeMeasurement1<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence, f64> for VectorLaplaceMechanism<T>
    where T: Copy + NumCast {
    fn make(sigma: f64) -> Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence> {
        let input_domain = VectorDomain::new_all();
        let output_domain = VectorDomain::new_all();
        let function = move |arg: &Vec<T>| -> Vec<T> {
            arg.iter()
                .map(|v| <f64 as NumCast>::from(*v).and_then(|v| T::from(v + laplace(sigma))))
                .collect::<Option<_>>().unwrap()
        };
        let input_metric = L1Sensitivity::new();
        let output_measure = MaxDivergence::new();
        let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
        Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}

pub struct GaussianMechanism<T> {
    data: PhantomData<T>
}

// gaussian for scalar-valued query
impl<T> MakeMeasurement1<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence, f64> for GaussianMechanism<T>
    where T: Copy + NumCast {
    fn make(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence> {
        let input_domain = AllDomain::new();
        let output_domain = AllDomain::new();
        let function = move |arg: &T| -> T {
            // TODO: switch to gaussian
            <f64 as NumCast>::from(*arg).and_then(|v| T::from(v + laplace(sigma))).unwrap()
        };
        let input_metric = L2Sensitivity::new();
        let output_measure = SmoothedMaxDivergence::new();
        // https://docs.google.com/spreadsheets/d/132rAzbSDVCKqFZWeE-P8oOl9f23PzkvNwsrDV5LPkw4/edit#gid=0
        let privacy_relation = move |d_in: &f64, d_out: &(f64, f64)| {
            let (eps, delta) = d_out.clone();
            eps.min(1.) >= (*d_in / sigma) * (2. * (1.25 / delta).ln()).sqrt()
        };
        Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_laplace_mechanism() {
        let measurement = LaplaceMechanism::<f64>::make(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }

    #[test]
    fn test_make_vector_laplace_mechanism() {
        let measurement = VectorLaplaceMechanism::<f64>::make(1.0);
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&1., &1.));
    }

    #[test]
    fn test_make_gaussian_mechanism() {
        let measurement = GaussianMechanism::<f64>::make(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);

        assert!(measurement.privacy_relation.eval(&0.1, &(0.5, 0.00001)));
    }
}
