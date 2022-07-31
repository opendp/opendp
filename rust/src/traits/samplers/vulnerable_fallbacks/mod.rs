

pub trait SampleDiscreteLaplaceZ2k: Sized {
    fn sample_discrete_laplace_Z2k(shift: Self, scale: Self, k: i32) -> Fallible<Self>;
}

impl<T> SampleDiscreteLaplaceZ2k for T
where
    T: num::Float
        + rand::distributions::uniform::SampleUniform
        + crate::traits::samplers::SampleRademacher,
{
    fn sample_discrete_laplace_Z2k(shift: Self, scale: Self, _k: i32) -> Fallible<Self> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut u: T = T::zero();
        while u.abs().is_zero() {
            u = rng.gen_range(T::from(-1.).unwrap(), T::from(1.).unwrap())
        }
        let value = shift + u.signum() * u.abs().ln() * scale;
        Ok(super::censor_neg_zero(value))
    }
}

pub trait SampleDiscreteGaussianZ2k: Sized {
    fn sample_discrete_gaussian_Z2k(shift: Self, scale: Self, k: i32) -> Fallible<Self>;
}

impl SampleDiscreteGaussianZ2k for f64 {
    fn sample_discrete_gaussian_Z2k(shift: Self, scale: Self, _k: i32) -> Fallible<Self> {
        use crate::traits::samplers::uniform::SampleUniform;
        let uniform_sample = f64::sample_standard_uniform(false)?;
        use statrs::function::erf;
        let value = shift + scale * std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * uniform_sample);
        Ok(censor_neg_zero(value))
    }
}

impl SampleDiscreteGaussianZ2k for f32 {
    fn sample_discrete_gaussian_Z2k(shift: Self, scale: Self, _k: i32) -> Fallible<Self> {
        use crate::traits::samplers::uniform::SampleUniform;
        let uniform_sample = f64::sample_standard_uniform(constant_time)?;
        use statrs::function::erf;
        let value = shift + scale * std::f32::consts::SQRT_2 * (erf::erfc_inv(2.0 * uniform_sample) as f32);
        Ok(censor_neg_zero(value))
    }
}