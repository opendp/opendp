//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use rand::Rng;

use crate::core::Measurement;
use crate::dist::{L2Sensitivity, L1Sensitivity, MaxDivergence, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use num::NumCast;

fn laplace(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5, 0.5);
    u.signum() * (1.0 - 2.0 * u.abs()).ln() * sigma
}

pub fn make_base_laplace<T>(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence> where
    T: Copy + NumCast {
    let input_domain = AllDomain::new();
    let output_domain = AllDomain::new();
    let function = move |arg: &T| -> T {
        // TODO: bubble up results from NumCast::from
        T::from(<f64 as NumCast>::from(*arg).unwrap() + laplace(sigma)).unwrap()
    };
    let input_metric = L1Sensitivity::new();
    let output_measure = MaxDivergence::new();
    let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
    Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
}

pub fn make_base_laplace_vec<T>(sigma: f64) -> Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence> where
    T: Copy + NumCast {

    let input_domain = VectorDomain::new(AllDomain::new());
    let output_domain = VectorDomain::new(AllDomain::new());
    let function = move |arg: &Vec<T>| -> Vec<T> {
        arg.iter().map(|v| T::from(<f64 as NumCast>::from(*v).unwrap() + laplace(sigma)).unwrap()).collect()
    };
    let input_metric = L1Sensitivity::new();
    let output_measure = MaxDivergence::new();
    let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
    Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
}

pub fn make_base_gaussian<T>(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence>
    where T: Copy + NumCast {
    let input_domain = AllDomain::new();
    let output_domain = AllDomain::new();
    let function = move |arg: &T| -> T {
        // TODO: switch to gaussian
        T::from(<f64 as NumCast>::from(*arg).unwrap() + laplace(sigma)).unwrap()
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_base_laplace() {
        let measurement = make_base_laplace::<f64>(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);
        // TODO: Test for base_laplace
    }

}
