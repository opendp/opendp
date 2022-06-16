use num::Zero;

use crate::error::Fallible;

use super::SampleRademacher;
#[cfg(feature="use-mpfr")]
use crate::traits::CastInternalReal;

#[cfg(not(feature="use-mpfr"))]
use rand::Rng;

#[cfg(feature="use-mpfr")]
use std::ops::Mul;

#[cfg(feature="use-mpfr")]
use rug::{Float, rand::ThreadRandState};

#[cfg(feature="use-mpfr")]
use super::GeneratorOpenDP;

pub trait SampleLaplace: SampleRademacher + Sized {
    fn sample_laplace(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self>;
}


pub trait SampleGaussian: Sized {
    /// Generates a draw from a Gaussian(loc, scale) distribution using the MPFR library.
    ///
    /// If shift = 0 and scale = 1, sampling is done in a way that respects exact rounding.
    /// Otherwise, the return will be the result of a composition of two operations that
    /// respect exact rounding (though the result will not necessarily).
    ///
    /// # Arguments
    /// * `shift` - The expectation of the Gaussian distribution.
    /// * `scale` - The scaling parameter (standard deviation) of the Gaussian distribution.
    /// * `constant_time` - Force underlying computations to run in constant time.
    ///
    /// # Return
    /// Draw from Gaussian(loc, scale)
    ///
    /// # Example
    /// ```
    /// use opendp::samplers::SampleGaussian;
    /// let gaussian = f64::sample_gaussian(0.0, 1.0, false);
    /// ```
    fn sample_gaussian(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self>;
}

/// If v is -0., return 0., otherwise return v.
/// This removes the duplicate -0. member of the output space,
/// which could hold an unintended bit of information
fn censor_neg_zero<T: Zero>(v: T) -> T {
    if v.is_zero() { T::zero() } else { v }
}

/// MPFR sets flags for [certain floating-point operations](https://docs.rs/gmp-mpfr-sys/1.4.7/gmp_mpfr_sys/C/MPFR/constant.MPFR_Interface.html#index-mpfr_005fclear_005fflags)
/// Clears all flags (underflow, overflow, divide-by-0, nan, inexact, erange).
#[cfg(feature="use-mpfr")]
fn censor_flags() {
    use gmp_mpfr_sys::mpfr::clear_flags;
    unsafe {clear_flags()}
}


/// Perturb `value` at a given `scale` using mean=0, scale=1 "exact" `noise`.
/// The general formula is: (shift / scale + noise) * scale
///
/// Floating-point arithmetic is performed with rounding such that
///     `scale` is a lower bound on the effective noise scale.
/// "exact" `noise` takes on any discrete representation in Float
///     with probability proportional to the analogous theoretical continuous distribution
///
/// To be valid, T::MANTISSA_BITS_U32 must be equal to the `noise` precision.
#[cfg(feature = "use-mpfr")]
fn perturb<T>(value: T, scale: T, noise: Float) -> T
    where T: Clone + CastInternalReal + Mul<Output=T> + Zero {

    use rug::float::Round;
    use rug::ops::{DivAssignRound, AddAssignRound};

    let mut value = value.into_internal();
    // when scaling into the noise coordinate space, round down so that noise is overestimated
    value.div_assign_round(&scale.clone().into_internal(), Round::Zero);
    // the noise itself is never scaled. Round away from zero to offset the scaling bias
    value.add_assign_round(
        &noise, if value.is_sign_positive() {Round::Up} else {Round::Down});
    // postprocess back to original coordinate space
    //     (remains differentially private via postprocessing)
    let value = T::from_internal(value) * scale;

    // clear all flags raised by mpfr to prevent side-channels
    censor_flags();

    // under no circumstance allow -0. to be returned
    // while exceedingly unlikely, if both the query and noise are -0., then the output is -0.,
    // which leaks that the input query was negatively signed.
    censor_neg_zero(value)
}

#[cfg(test)]
mod test_mpfr {
    use rug::Float;
    use gmp_mpfr_sys::mpfr::{inexflag_p, clear_inexflag, underflow_p, clear_underflow};
    use std::ops::MulAssign;

    #[test]
    fn test_neg_zero() {
        let a = Float::with_val(53, -0.0);
        let b = Float::with_val(53, -0.0);
        // neg zero is propagated
        assert!((a + b).is_sign_negative());
    }
    #[test]
    fn test_inexflag() {
        println!("inexflag before:  {:?}", unsafe {inexflag_p()});
        let a = Float::with_val(53, 0.1);
        let b = Float::with_val(53, 0.2);
        let _ = a + b;

        println!("inexflag after:   {:?}", unsafe {inexflag_p()});
        unsafe {clear_inexflag()}

        println!("inexflag cleared: {:?}", unsafe {inexflag_p()});
    }

    #[test]
    fn test_underflow_flag() {
        println!("flag before:       {:?}", unsafe {underflow_p()});
        // taking advantage of subnormal representation, which is smaller than f64::MIN_POSITIVE
        let smallest_float = f64::from_bits(1);
        println!("smallest float:    {:e}", smallest_float);
        println!("underflow float:   {:e}", smallest_float / 2.);
        let mut a = Float::with_val(53, smallest_float);
        println!("smallest rug?:     {:?}", a);
        // somehow rug represents numbers beyond the given precision
        println!("smaller rug:       {:?}", a.clone() / 2.);
        // tetrate to force underflow
        for _ in 0..32 { a.mul_assign(&a.clone()); }
        println!("underflow rug:     {:?}", a);

        println!("flag after:        {:?}", unsafe {underflow_p()});
        unsafe {clear_underflow()}

        println!("flag cleared:      {:?}", unsafe {underflow_p()});
    }
}


#[cfg(feature = "use-mpfr")]
impl<T: Clone + CastInternalReal + SampleRademacher + Zero + Mul<Output=T>> SampleLaplace for T {
    fn sample_laplace(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        if scale.is_zero() { return Ok(shift) }
        if constant_time {
            return fallible!(FailedFunction, "mpfr samplers do not support constant time execution")
        }

        // initialize randomness
        let mut rng = GeneratorOpenDP::new();
        let laplace = {
            let mut state = ThreadRandState::new_custom(&mut rng);

            // see https://arxiv.org/pdf/1303.6257.pdf, algorithm V for exact standard exponential deviates
            let exponential = rug::Float::with_val(
                Self::MANTISSA_DIGITS, rug::Float::random_exp(&mut state));
            // adding a random sign to the exponential deviate does not induce gaps or stacks
            exponential * T::sample_standard_rademacher()?.into_internal()
        };
        rng.error?;

        Ok(perturb(shift, scale, laplace))
    }
}

#[cfg(not(feature = "use-mpfr"))]
impl<T: num::Float + rand::distributions::uniform::SampleUniform + SampleRademacher> SampleLaplace for T {
    fn sample_laplace(shift: Self, scale: Self, _constant_time: bool) -> Fallible<Self> {
        let mut rng = rand::thread_rng();
        let mut u: T = T::zero();
        while u.abs().is_zero() {
            u = rng.gen_range(T::from(-1.).unwrap(), T::from(1.).unwrap())
        }
        let value = shift + u.signum() * u.abs().ln() * scale;
        Ok(censor_neg_zero(value))
    }
}

#[cfg(feature = "use-mpfr")]
impl<T: Clone + CastInternalReal + Zero + Mul<Output=T>> SampleGaussian for T {

    fn sample_gaussian(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        if scale.is_zero() { return Ok(shift) }
        if constant_time {
            return fallible!(FailedFunction, "mpfr samplers do not support constant time execution")
        }

        // initialize randomness
        let mut rng = GeneratorOpenDP::new();
        let gauss = {
            let mut state = ThreadRandState::new_custom(&mut rng);

            // generate Gaussian(0,1) according to mpfr standard
            // See https://arxiv.org/pdf/1303.6257.pdf, algorithm N for exact standard normal deviates
            rug::Float::with_val(Self::MANTISSA_DIGITS, Float::random_normal(&mut state))
        };
        rng.error?;

        Ok(perturb(shift, scale, gauss))
    }
}


#[cfg(not(feature = "use-mpfr"))]
impl SampleGaussian for f64 {
    fn sample_gaussian(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        use crate::traits::samplers::uniform::SampleUniform;
        let uniform_sample = f64::sample_standard_uniform(constant_time)?;
        use statrs::function::erf;
        let value = shift + scale * std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * uniform_sample);
        Ok(censor_neg_zero(value))
    }
}

#[cfg(not(feature = "use-mpfr"))]
impl SampleGaussian for f32 {
    fn sample_gaussian(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        use crate::traits::samplers::uniform::SampleUniform;
        let uniform_sample = f64::sample_standard_uniform(constant_time)?;
        use statrs::function::erf;
        let value = shift + scale * std::f32::consts::SQRT_2 * (erf::erfc_inv(2.0 * uniform_sample) as f32);
        Ok(censor_neg_zero(value))
    }
}




#[cfg(test)]
mod test_utils {
    #[cfg(feature="test-plot")]
    fn plot_continuous(title: String, data: Vec<f64>) -> Fallible<()> {
        use vega_lite_4::*;

        VegaliteBuilder::default()
            .title(title)
            .data(&data)
            .mark(Mark::Area)
            .transform(vec![TransformBuilder::default().density("data").build()?])
            .encoding(
                EdEncodingBuilder::default()
                    .x(XClassBuilder::default()
                        .field("value")
                        .position_def_type(Type::Quantitative)
                        .build()?)
                    .y(YClassBuilder::default()
                        .field("density")
                        .position_def_type(Type::Quantitative)
                        .build()?)
                    .build()?,
            )
            .build()?.show().unwrap_test();
        Ok(())
    }

    #[test]
    #[cfg(feature="test-plot")]
    fn plot_laplace() -> Fallible<()> {
        let shift = 0.;
        let scale = 5.;

        let title = format!("Laplace(shift={}, scale={}) distribution", shift, scale);
        let data = (0..10_000)
            .map(|_| f64::sample_laplace(shift, scale, false))
            .collect::<Fallible<Vec<f64>>>()?;

        plot_continuous(title, data).unwrap_test();
        Ok(())
    }


    #[test]
    #[cfg(feature="test-plot")]
    fn plot_gaussian() -> Fallible<()> {
        let shift = 0.;
        let scale = 5.;

        let title = format!("Gaussian(shift={}, scale={}) distribution", shift, scale);
        let data = (0..10_000)
            .map(|_| f64::sample_gaussian(shift, scale, false))
            .collect::<Fallible<Vec<f64>>>()?;

        plot_continuous(title, data).unwrap_test();
        Ok(())
    }
}