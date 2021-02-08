//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use rand::Rng;

use crate::core::Measurement;
use crate::dist::{L2Sensitivity, L1Sensitivity, MaxDivergence, SmoothedMaxDivergence};
use crate::dom::{AllDomain, VectorDomain};

fn laplace(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5, 0.5);
    u.signum() * (1.0 - 2.0 * u.abs()).ln() * sigma
}

pub trait AddNoise {
    fn add_noise(self, noise: f64) -> Self;
}
impl AddNoise for u32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u8 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }


pub trait OpendpInto<T> {
    fn opendp_into(self) -> T;
}


impl OpendpInto<u32> for u32 { fn opendp_into(self) -> u32 { self as u32 } }

impl OpendpInto<u32> for u64 { fn opendp_into(self) -> u32 { self as u32 } }

impl OpendpInto<u32> for i32 { fn opendp_into(self) -> u32 { self as u32 } }

impl OpendpInto<u32> for i64 { fn opendp_into(self) -> u32 { self as u32 } }

impl OpendpInto<u32> for f32 { fn opendp_into(self) -> u32 { self as u32 } }

impl OpendpInto<u32> for f64 { fn opendp_into(self) -> u32 { self as u32 } }

impl OpendpInto<u32> for u8 { fn opendp_into(self) -> u32 { self as u32 } }

impl OpendpInto<u64> for u32 { fn opendp_into(self) -> u64 { self as u64 } }

impl OpendpInto<u64> for u64 { fn opendp_into(self) -> u64 { self as u64 } }

impl OpendpInto<u64> for i32 { fn opendp_into(self) -> u64 { self as u64 } }

impl OpendpInto<u64> for i64 { fn opendp_into(self) -> u64 { self as u64 } }

impl OpendpInto<u64> for f32 { fn opendp_into(self) -> u64 { self as u64 } }

impl OpendpInto<u64> for f64 { fn opendp_into(self) -> u64 { self as u64 } }

impl OpendpInto<u64> for u8 { fn opendp_into(self) -> u64 { self as u64 } }

impl OpendpInto<i32> for u32 { fn opendp_into(self) -> i32 { self as i32 } }

impl OpendpInto<i32> for u64 { fn opendp_into(self) -> i32 { self as i32 } }

impl OpendpInto<i32> for i32 { fn opendp_into(self) -> i32 { self as i32 } }

impl OpendpInto<i32> for i64 { fn opendp_into(self) -> i32 { self as i32 } }

impl OpendpInto<i32> for f32 { fn opendp_into(self) -> i32 { self as i32 } }

impl OpendpInto<i32> for f64 { fn opendp_into(self) -> i32 { self as i32 } }

impl OpendpInto<i32> for u8 { fn opendp_into(self) -> i32 { self as i32 } }

impl OpendpInto<i64> for u32 { fn opendp_into(self) -> i64 { self as i64 } }

impl OpendpInto<i64> for u64 { fn opendp_into(self) -> i64 { self as i64 } }

impl OpendpInto<i64> for i32 { fn opendp_into(self) -> i64 { self as i64 } }

impl OpendpInto<i64> for i64 { fn opendp_into(self) -> i64 { self as i64 } }

impl OpendpInto<i64> for f32 { fn opendp_into(self) -> i64 { self as i64 } }

impl OpendpInto<i64> for f64 { fn opendp_into(self) -> i64 { self as i64 } }

impl OpendpInto<i64> for u8 { fn opendp_into(self) -> i64 { self as i64 } }

impl OpendpInto<f32> for u32 { fn opendp_into(self) -> f32 { self as f32 } }

impl OpendpInto<f32> for u64 { fn opendp_into(self) -> f32 { self as f32 } }

impl OpendpInto<f32> for i32 { fn opendp_into(self) -> f32 { self as f32 } }

impl OpendpInto<f32> for i64 { fn opendp_into(self) -> f32 { self as f32 } }

impl OpendpInto<f32> for f32 { fn opendp_into(self) -> f32 { self as f32 } }

impl OpendpInto<f32> for f64 { fn opendp_into(self) -> f32 { self as f32 } }

impl OpendpInto<f32> for u8 { fn opendp_into(self) -> f32 { self as f32 } }

impl OpendpInto<f64> for u32 { fn opendp_into(self) -> f64 { self as f64 } }

impl OpendpInto<f64> for u64 { fn opendp_into(self) -> f64 { self as f64 } }

impl OpendpInto<f64> for i32 { fn opendp_into(self) -> f64 { self as f64 } }

impl OpendpInto<f64> for i64 { fn opendp_into(self) -> f64 { self as f64 } }

impl OpendpInto<f64> for f32 { fn opendp_into(self) -> f64 { self as f64 } }

impl OpendpInto<f64> for f64 { fn opendp_into(self) -> f64 { self as f64 } }

impl OpendpInto<f64> for u8 { fn opendp_into(self) -> f64 { self as f64 } }

impl OpendpInto<u8> for u32 { fn opendp_into(self) -> u8 { self as u8 } }

impl OpendpInto<u8> for u64 { fn opendp_into(self) -> u8 { self as u8 } }

impl OpendpInto<u8> for i32 { fn opendp_into(self) -> u8 { self as u8 } }

impl OpendpInto<u8> for i64 { fn opendp_into(self) -> u8 { self as u8 } }

impl OpendpInto<u8> for f32 { fn opendp_into(self) -> u8 { self as u8 } }

impl OpendpInto<u8> for f64 { fn opendp_into(self) -> u8 { self as u8 } }

impl OpendpInto<u8> for u8 { fn opendp_into(self) -> u8  { self as u8 }}


pub fn make_base_laplace<T>(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence> where
    T: Copy + OpendpInto<f64>,
    f64: OpendpInto<T> {
    let input_domain = AllDomain::new();
    let output_domain = AllDomain::new();
    let function = move |arg: &T| -> T {
        f64::opendp_into(T::opendp_into(*arg) + laplace(sigma))
    };
    let input_metric = L1Sensitivity::new();
    let output_measure = MaxDivergence::new();
    let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
    Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
}

pub fn make_base_laplace_vec<T>(sigma: f64) -> Measurement<VectorDomain<AllDomain<T>>, VectorDomain<AllDomain<T>>, L1Sensitivity<f64>, MaxDivergence> where
    T: Copy + OpendpInto<f64>,
    f64: OpendpInto<T> {

    let input_domain = VectorDomain::new(AllDomain::new());
    let output_domain = VectorDomain::new(AllDomain::new());
    let function = move |arg: &Vec<T>| -> Vec<T> {
        arg.iter().map(|v| f64::opendp_into(T::opendp_into(*v) + laplace(sigma))).collect()
    };
    let input_metric = L1Sensitivity::new();
    let output_measure = MaxDivergence::new();
    let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
    Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
}

pub fn make_base_gaussian<T>(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L2Sensitivity<f64>, SmoothedMaxDivergence>
    where T: Copy + AddNoise {
    let input_domain = AllDomain::new();
    let output_domain = AllDomain::new();
    let function = move |arg: &T| -> T {
        // TODO: switch to gaussian
        let noise = laplace(sigma);
        arg.add_noise(noise)
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
