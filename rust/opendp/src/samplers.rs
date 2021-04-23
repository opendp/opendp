use std::cmp;
use std::ops::{AddAssign, Neg};

use ieee754::Ieee754;

use num::{One, Zero};
use openssl::rand::rand_bytes;
#[cfg(feature="use-mpfr")]
use rug::{Float, rand::{ThreadRandGen, ThreadRandState}};


pub fn fill_bytes(mut buffer: &mut [u8]) -> Fallible<()> {
    if let Err(e) = rand_bytes(&mut buffer) {
        fallible!(FailedFunction, "OpenSSL error: {:?}", e)
    } else { Ok(()) }
}

use crate::error::Fallible;
#[cfg(not(feature="use-mpfr"))]
use statrs::function::erf;
#[cfg(not(feature="use-mpfr"))]
use rand::Rng;

#[cfg(feature="use-mpfr")]
pub(crate) struct GeneratorOpenSSL;

#[cfg(feature="use-mpfr")]
impl ThreadRandGen for GeneratorOpenSSL {
    fn gen(&mut self) -> u32 {
        let mut buffer = [0u8; 4];
        // impossible not to panic here
        //    cannot ignore errors with .ok(), because the buffer will remain 0
        fill_bytes(&mut buffer).unwrap();
        u32::from_ne_bytes(buffer)
    }
}


// SAMPLERS
pub trait SampleBernoulli: Sized {
    fn sample_standard_bernoulli() -> Fallible<Self>;

    /// Sample a single bit with arbitrary probability of success
    ///
    /// Uses only an unbiased source of coin flips.
    /// The strategy for doing this with 2 flips in expectation is described [here](https://web.archive.org/web/20160418185834/https://amakelov.wordpress.com/2013/10/10/arbitrarily-biasing-a-coin-in-2-expected-tosses/).
    ///
    /// # Arguments
    /// * `prob`- The desired probability of success (bit = 1).
    /// * `enforce_constant_time` - Whether or not to enforce the algorithm to run in constant time
    ///
    /// # Return
    /// A bit that is 1 with probability "prob"
    ///
    /// # Examples
    ///
    /// ```
    /// // returns a bit with Pr(bit = 1) = 0.7
    /// use opendp::samplers::SampleBernoulli;
    /// let n = bool::sample_bernoulli(0.7, false);
    /// # use opendp::error::ExplainUnwrap;
    /// # n.unwrap_test();
    /// ```
    /// ```should_panic
    /// // fails because 1.3 not a valid probability
    /// use opendp::samplers::SampleBernoulli;
    /// let n = bool::sample_bernoulli(1.3, false);
    /// # use opendp::error::ExplainUnwrap;
    /// # n.unwrap_test();
    /// ```
    /// ```should_panic
    /// // fails because -0.3 is not a valid probability
    /// use opendp::samplers::SampleBernoulli;
    /// let n = bool::sample_bernoulli(-0.3, false);
    /// # use opendp::error::ExplainUnwrap;
    /// # n.unwrap_test();
    /// ```
    fn sample_bernoulli(prob: f64, enforce_constant_time: bool) -> Fallible<Self>;
}

impl SampleBernoulli for bool {
    fn sample_standard_bernoulli() -> Fallible<Self> {
        let mut buffer = [0u8; 1];
        fill_bytes(&mut buffer)?;
        Ok(buffer[0] & 1 == 1)
    }

    fn sample_bernoulli(prob: f64, enforce_constant_time: bool) -> Fallible<Self> {

        // ensure that prob is a valid probability
        if prob < 0.0 || prob > 1.0 {return fallible!(FailedFunction, "probability is not within [0, 1]")}

        // decompose probability into mantissa and exponent integers to quickly identify the value in the first_heads_index
        let (_sign, exponent, mantissa) = prob.decompose_raw();

        // repeatedly flip fair coin (up to 1023 times) and identify index (0-based) of first heads
        let first_heads_index = sample_i10_geometric(enforce_constant_time)?;

        // if prob == 1., return after retrieving censored_specific_geom, to protect constant time
        if exponent == 1023 { return Ok(true) }

        // number of leading zeros in binary representation of prob
        //    cast is non-saturating because exponent only uses first 11 bits
        //    exponent is bounded within [0, 1022] by check for valid probability
        let num_leading_zeros = 1022_i16 - exponent as i16;

        // 0 is the most significant/leftmost implicit bit in the mantissa/fraction/significand
        // 52 is the least significant/rightmost
        Ok(match first_heads_index - num_leading_zeros {
            // index into the leading zeros of the binary representation
            i if i < 0 => false,
            // bit index 0 is implicitly set in ieee-754 when the exponent is nonzero
            i if i == 0 => exponent != 0,
            // all other digits out-of-bounds are not float-approximated/are-implicitly-zero
            i if i > 52 => false,
            // retrieve the bit at `i` slots shifted from the left
            i => mantissa & (1_u64 << (52 - i as usize)) != 0
        })
    }
}

pub trait SampleRademacher: Sized {
    fn sample_standard_rademacher() -> Fallible<Self>;
    fn sample_rademacher(prob: f64, enforce_constant_time: bool) -> Fallible<Self>;
}

impl<T: Neg<Output=T> + One> SampleRademacher for T {
    fn sample_standard_rademacher() -> Fallible<Self> {
        Ok(if bool::sample_standard_bernoulli()? {T::one()} else {T::one().neg()})
    }
    fn sample_rademacher(prob: f64, enforce_constant_time: bool) -> Fallible<Self> {
        Ok(if bool::sample_bernoulli(prob, enforce_constant_time)? {T::one()} else {T::one().neg()})
    }
}

pub trait SampleUniform: Sized {

    /// Returns a random sample from Uniform[0,1).
    ///
    /// This algorithm is taken from [Mironov (2012)](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.366.5957&rep=rep1&type=pdf)
    /// and is important for making some of the guarantees in the paper.
    ///
    /// The idea behind the uniform sampling is to first sample a "precision band".
    /// Each band is a range of floating point numbers with the same level of arithmetic precision
    /// and is situated between powers of two.
    /// A band is sampled with probability relative to the unit of least precision using the Geometric distribution.
    /// That is, the uniform sampler will generate the band [1/2,1) with probability 1/2, [1/4,1/2) with probability 1/4,
    /// and so on.
    ///
    /// Once the precision band has been selected, floating numbers numbers are generated uniformly within the band
    /// by generating a 52-bit mantissa uniformly at random.
    ///
    /// # Arguments
    ///
    /// `min`: f64 minimum of uniform distribution (inclusive)
    /// `max`: f64 maximum of uniform distribution (non-inclusive)
    ///
    /// # Return
    /// Random draw from Unif[min, max).
    ///
    /// # Example
    /// ```
    /// // valid draw from Unif[0,1)
    /// use opendp::samplers::SampleUniform;
    /// let unif = f64::sample_standard_uniform(false);
    /// # use opendp::error::ExplainUnwrap;
    /// # unif.unwrap_test();
    /// ```
    fn sample_standard_uniform(enforce_constant_time: bool) -> Fallible<Self>;
}

impl SampleUniform for f64 {
    fn sample_standard_uniform(enforce_constant_time: bool) -> Fallible<Self> {

        // A saturated mantissa with implicit bit is ~2
        let exponent: i16 = -(1 + sample_i10_geometric(enforce_constant_time)?);

        let mantissa: u64 = {
            let mut mantissa_buffer = [0u8; 8];
            // mantissa bit index zero is implicit
            fill_bytes(&mut mantissa_buffer[1..])?;
            // limit the buffer to 52 bits
            mantissa_buffer[1] %= 16;

            // convert mantissa to integer
            u64::from_be_bytes(mantissa_buffer)
        };

        // Generate uniform random number from [0,1)
        Ok(Self::recompose(false, exponent, mantissa))
    }
}

impl SampleUniform for f32 {
    fn sample_standard_uniform(enforce_constant_time: bool) -> Fallible<Self> {
        f64::sample_standard_uniform(enforce_constant_time).map(|v| v as f32)
    }
}

/// Return sample from a censored Geometric distribution with parameter p=0.5 without calling to sample_bit_prob.
///
/// The algorithm generates 1023 bits uniformly at random and returns the
/// index of the first bit with value 1. If all 1023 bits are 0, then
/// the algorithm acts as if the last bit was a 1 and returns 1022.
///
/// This is a less general version of the sample_geometric function.
/// The major difference is that this function does not
/// call sample_geometric itself (whereas sample_geometric does), so having this more specialized
/// version allows us to avoid an infinite dependence loop.
fn sample_i10_geometric(enforce_constant_time: bool) -> Fallible<i16> {
    Ok(if enforce_constant_time {
        let mut buffer = vec![0_u8; 128];
        fill_bytes(&mut buffer)?;

        cmp::min(buffer.into_iter().enumerate()
                     // ignore samples that contain no events
                     .filter(|(_, sample)| sample > &0)
                     // compute the index of the smallest event in the batch
                     .map(|(i, sample)| 8 * i + sample.leading_zeros() as usize)
                     // retrieve the smallest index
                     .min()
                     // return 1022 if no events occurred (slight dp violation w.p. ~2^-52)
                     .unwrap_or(1022) as i16, 1022)

    } else {
        // retrieve up to 128 bytes, each containing 8 trials
        for i in 0..128 {
            let mut buffer = vec![0_u8; 1];
            fill_bytes(&mut buffer)?;

            if buffer[0] > 0 {
                return Ok(cmp::min(i * 8 + buffer[0].leading_zeros() as i16, 1022))
            }
        }
        1022
    })
}


pub trait SampleGeometric: Sized {

    /// Sample from the censored geometric distribution with parameter "prob" and maximum
    /// number of trials "max_trials".
    ///
    /// # Arguments
    /// * `prob` - Parameter for the geometric distribution, the probability of success on any given trials.
    /// * `max_trials` - The maximum number of trials allowed.
    /// * `enforce_constant_time` - Whether or not to enforce the algorithm to run in constant time; if true,
    ///                             it will always run for "max_trials" trials.
    ///
    /// # Return
    /// A draw from the censored geometric distribution.
    ///
    /// # Example
    /// ```
    /// use opendp::samplers::SampleGeometric;
    /// let geom = u8::sample_geometric(0.1, 20, false);
    /// # use opendp::error::ExplainUnwrap;
    /// # geom.unwrap_test();
    /// ```
    fn sample_geometric(prob: f64, max_trials: Self, enforce_constant_time: bool) -> Fallible<Self>;
}

impl<T: Zero + One + PartialOrd + AddAssign + Clone> SampleGeometric for T {

    fn sample_geometric(prob: f64, max_trials: Self, enforce_constant_time: bool) -> Fallible<Self> {

        // ensure that prob is a valid probability
        if prob < 0.0 || prob > 1.0 {return fallible!(FailedFunction, "probability is not within [0, 1]")}

        let mut n_trials: Self = T::zero();
        let mut geom_return: Self = T::zero();

        // generate bits until we find a 1
        // if enforcing the runtime of the algorithm to be constant, the while loop
        // continues after the 1 is found and just stores the first location of a 1 bit.
        while n_trials < max_trials {
            n_trials += T::one();

            // If we haven't seen a 1 yet, set the return to the current number of trials
            if bool::sample_bernoulli(prob, enforce_constant_time)? && geom_return.is_zero() {
                geom_return = n_trials.clone();
                if !enforce_constant_time {
                    return Ok(geom_return);
                }
            }
        }

        // set geom_return to max if we never saw a bit equaling 1
        if geom_return.is_zero() {
            geom_return = max_trials; // could also set this equal to n_trials - 1.
        }

        Ok(geom_return)
    }
}


pub trait SampleLaplace: SampleRademacher + Sized {
    fn sample_laplace(shift: Self, scale: Self, enforce_constant_time: bool) -> Fallible<Self>;
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
    /// * `enforce_constant_time` - Force underlying computations to run in constant time.
    ///
    /// # Return
    /// Draw from Gaussian(loc, scale)
    ///
    /// # Example
    /// ```
    /// use opendp::samplers::SampleGaussian;
    /// let gaussian = f64::sample_gaussian(0.0, 1.0, false);
    /// ```
    fn sample_gaussian(shift: Self, scale: Self, enforce_constant_time: bool) -> Fallible<Self>;
}


pub trait MantissaDigits { const MANTISSA_DIGITS: u32; }

impl MantissaDigits for f32 { const MANTISSA_DIGITS: u32 = f32::MANTISSA_DIGITS; }

impl MantissaDigits for f64 { const MANTISSA_DIGITS: u32 = f64::MANTISSA_DIGITS; }

#[cfg(feature = "use-mpfr")]
pub trait CastRug: MantissaDigits + Sized {
    fn from_rug(v: Float) -> Self;
    fn into_rug(self) -> Float;
}

#[cfg(feature = "use-mpfr")]
impl CastRug for f64 {
    fn from_rug(v: Float) -> Self { v.to_f64() }
    fn into_rug(self) -> Float { rug::Float::with_val(Self::MANTISSA_DIGITS, self) }
}

#[cfg(feature = "use-mpfr")]
impl CastRug for f32 {
    fn from_rug(v: Float) -> Self { v.to_f32() }
    fn into_rug(self) -> Float { rug::Float::with_val(Self::MANTISSA_DIGITS, self) }
}

#[cfg(feature = "use-mpfr")]
impl<T: CastRug + SampleRademacher> SampleLaplace for T {
    fn sample_laplace(shift: Self, scale: Self, enforce_constant_time: bool) -> Fallible<Self> {
        if enforce_constant_time {
            return fallible!(FailedFunction, "mpfr samplers do not support constant time execution")
        }

        let shift = shift.into_rug();
        let scale = scale.into_rug() * T::sample_standard_rademacher()?.into_rug();
        let standard_exponential_sample = {
            let mut rng = GeneratorOpenSSL {};
            let mut state = ThreadRandState::new_custom(&mut rng);
            rug::Float::with_val(Self::MANTISSA_DIGITS, rug::Float::random_exp(&mut state))
        };

        Ok(Self::from_rug(standard_exponential_sample.mul_add(&scale, &shift)))
    }
}

#[cfg(not(feature = "use-mpfr"))]
impl<T: num::Float + rand::distributions::uniform::SampleUniform + SampleRademacher> SampleLaplace for T {
    fn sample_laplace(shift: Self, scale: Self, _enforce_constant_time: bool) -> Fallible<Self> {
        let mut rng = rand::thread_rng();
        let _1_ = T::from(1.0).unwrap();
        let _2_ = T::from(2.0).unwrap();
        let u: T = rng.gen_range(T::from(-0.5).unwrap(), T::from(0.5).unwrap());
        Ok(shift - u.signum() * (_1_ - _2_ * u.abs()).ln() * scale)
    }
}

#[cfg(feature = "use-mpfr")]
impl<T: CastRug> SampleGaussian for T {

    fn sample_gaussian(shift: Self, scale: Self, enforce_constant_time: bool) -> Fallible<Self> {
        if enforce_constant_time {
            return fallible!(FailedFunction, "mpfr samplers do not support constant time execution")
        }

        // initialize randomness
        let mut rng = GeneratorOpenSSL {};
        let mut state = ThreadRandState::new_custom(&mut rng);

        // generate Gaussian(0,1) according to mpfr standard
        let gauss = rug::Float::with_val(Self::MANTISSA_DIGITS, Float::random_normal(&mut state));

        // initialize floats within mpfr/rug
        let shift = shift.into_rug();
        let scale = scale.into_rug();
        Ok(Self::from_rug(gauss.mul_add(&scale, &shift)))
    }
}


#[cfg(not(feature = "use-mpfr"))]
impl SampleGaussian for f64 {
    fn sample_gaussian(shift: Self, scale: Self, enforce_constant_time: bool) -> Fallible<Self> {
        let uniform_sample = f64::sample_standard_uniform(enforce_constant_time)?;
        Ok(shift + scale * std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * uniform_sample))
    }
}

#[cfg(not(feature = "use-mpfr"))]
impl SampleGaussian for f32 {
    fn sample_gaussian(shift: Self, scale: Self, enforce_constant_time: bool) -> Fallible<Self> {
        let uniform_sample = f64::sample_standard_uniform(enforce_constant_time)?;
        Ok(shift + scale * std::f32::consts::SQRT_2 * (erf::erfc_inv(2.0 * uniform_sample) as f32))
    }
}