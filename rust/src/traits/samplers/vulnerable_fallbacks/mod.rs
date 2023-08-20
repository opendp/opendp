use crate::error::Fallible;
use crate::traits::samplers::{SampleStandardBernoulli, SampleUniform};
use statrs::function::erf;

pub trait SampleDiscreteLaplaceZ2k: Sized {
    fn sample_discrete_laplace_Z2k(shift: Self, scale: Self, k: i32) -> Fallible<Self>;
}

impl<T> SampleDiscreteLaplaceZ2k for T
where
    T: num::Float + SampleUniform,
{
    fn sample_discrete_laplace_Z2k(shift: Self, scale: Self, _k: i32) -> Fallible<Self> {
        let u = loop {
            let u = T::sample_standard_uniform(false)?;
            if !u.is_zero() {
                break if bool::sample_standard_bernoulli()? {
                    u
                } else {
                    -u
                };
            }
        };
        Ok(shift + u.signum() * u.abs().ln() * scale)
    }
}

pub trait SampleDiscreteGaussianZ2k: Sized {
    fn sample_discrete_gaussian_Z2k(shift: Self, scale: Self, k: i32) -> Fallible<Self>;
}

impl SampleDiscreteGaussianZ2k for f64 {
    fn sample_discrete_gaussian_Z2k(shift: Self, scale: Self, _k: i32) -> Fallible<Self> {
        let uniform_sample = f64::sample_standard_uniform(false)?;
        Ok(shift + scale * std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * uniform_sample))
    }
}

impl SampleDiscreteGaussianZ2k for f32 {
    fn sample_discrete_gaussian_Z2k(shift: Self, scale: Self, _k: i32) -> Fallible<Self> {
        let uniform_sample = f64::sample_standard_uniform(false)?;
        Ok(shift + scale * std::f32::consts::SQRT_2 * (erf::erfc_inv(2.0 * uniform_sample) as f32))
    }
}

pub trait CastInternalRational {}
impl<T> CastInternalRational for T {}
