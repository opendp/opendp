//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use std::cmp;

use ieee754::Ieee754;
use openssl::rand::rand_bytes;
#[cfg(not(feature="use-mpfr"))]
use rand::Rng;
#[cfg(feature="use-mpfr")]
use rug::{Float, rand::{ThreadRandGen, ThreadRandState}};
#[cfg(not(feature="use-mpfr"))]
use statrs::function::erf;

use crate::core::{Domain, Measure, Metric};

pub mod laplace;
pub mod gaussian;

// Trait for all constructors, can have different implementations depending on concrete types of Domains and/or Metrics
pub trait MakeMeasurement<DI: Domain, DO: Domain, MI: Metric, MO: Measure> {
    fn make() -> crate::core::Measurement<DI, DO, MI, MO> {
        Self::make0()
    }
    fn make0() -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement1<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1> {
    fn make(param1: P1) -> crate::core::Measurement<DI, DO, MI, MO> {
        Self::make1(param1)
    }
    fn make1(param1: P1) -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement2<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2> {
    fn make(param1: P1, param2: P2) -> crate::core::Measurement<DI, DO, MI, MO> {
        Self::make2(param1, param2)
    }
    fn make2(param1: P1, param2: P2) -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement3<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3> {
    fn make(param1: P1, param2: P2, param3: P3) -> crate::core::Measurement<DI, DO, MI, MO> {
        Self::make3(param1, param2, param3)
    }
    fn make3(param1: P1, param2: P2, param3: P3) -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub trait MakeMeasurement4<DI: Domain, DO: Domain, MI: Metric, MO: Measure, P1, P2, P3, P4> {
    fn make(param1: P1, param2: P2, param3: P3, param4: P4) -> crate::core::Measurement<DI, DO, MI, MO> {
        Self::make4(param1, param2, param3, param4)
    }
    fn make4(param1: P1, param2: P2, param3: P3, param4: P4) -> crate::core::Measurement<DI, DO, MI, MO>;
}

pub fn fill_bytes(mut buffer: &mut [u8]) -> Result<(), &'static str> {
    if let Err(_e) = rand_bytes(&mut buffer) {
        Err("OpenSSL Error")
    } else { Ok(()) }
}

#[cfg(feature="use-mpfr")]
struct GeneratorOpenSSL;

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
pub fn sample_standard_bernoulli() -> Result<bool, &'static str> {
    let mut buffer = [0u8; 1];
    fill_bytes(&mut buffer)?;
    Ok(buffer[0] & 1 == 1)
}


/// Sample a single bit with arbitrary probability of success
///
/// Uses only an unbiased source of coin flips.
/// The strategy for doing this with 2 flips in expectation is described [here](https://amakelov.wordpress.com/2013/10/10/arbitrarily-biasing-a-coin-in-2-expected-tosses/).
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
/// use opendp::meas::sample_bernoulli;
/// let n = sample_bernoulli(0.7, false);
/// # n.unwrap();
/// ```
/// ```should_panic
/// // fails because 1.3 not a valid probability
/// use opendp::meas::sample_bernoulli;
/// let n = sample_bernoulli(1.3, false);
/// # n.unwrap();
/// ```
/// ```should_panic
/// // fails because -0.3 is not a valid probability
/// use opendp::meas::sample_bernoulli;
/// let n = sample_bernoulli(-0.3, false);
/// # n.unwrap();
/// ```
pub fn sample_bernoulli(prob: f64, enforce_constant_time: bool) -> Result<bool, &'static str> {

    // ensure that prob is a valid probability
    if prob < 0.0 || prob > 1.0 {return Err("probability is not within [0, 1]")}

    // decompose probability into mantissa and exponent integers to quickly identify the value in the first_heads_index
    let (_sign, exponent, mantissa) = prob.decompose_raw();

    // repeatedly flip fair coin (up to 1023 times) and identify index (0-based) of first heads
    let first_heads_index = sample_censored_standard_geometric(enforce_constant_time)?;

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


/// Returns random sample from Uniform[0,1).
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
/// use opendp::meas::sample_standard_uniform;
/// let unif = sample_standard_uniform(false);
/// # unif.unwrap();
/// ```
pub fn sample_standard_uniform(enforce_constant_time: bool) -> Result<f64, &'static str> {

    // Generate mantissa
    let mut mantissa_buffer = [0u8; 8];
    // mantissa bit index zero is implicit
    fill_bytes(&mut mantissa_buffer[1..])?;
    // limit the buffer to 52 bits
    mantissa_buffer[1] %= 16;

    // convert mantissa to integer
    let mantissa_int = u64::from_be_bytes(mantissa_buffer);

    // Generate exponent. A saturated mantissa with implicit bit is ~2
    let exponent: i16 = -(1 + sample_censored_standard_geometric(enforce_constant_time)?);

    // Generate uniform random number from [0,1)
    let uniform_rand = f64::recompose(false, exponent, mantissa_int);

    Ok(uniform_rand)
}


/// Return sample from a censored Geometric distribution with parameter p=0.5 without calling to sample_bit_prob.
///
/// The algorithm generates 1023 bits uniformly at random and returns the
/// index of the first bit with value 1. If all 1023 bits are 0, then
/// the algorithm acts as if the last bit was a 1 and returns 1022.
///
/// This is a less general version of the sample_geometric function, designed to be used
/// only inside of the sample_bit_prob function. The major difference is that this function does not
/// call sample_bit_prob itself (whereas sample_geometric does), so having this more specialized
/// version allows us to avoid an infinite dependence loop.
pub fn sample_censored_standard_geometric(enforce_constant_time: bool) -> Result<i16, &'static str> {

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
/// use opendp::meas::sample_censored_geometric;
/// let geom = sample_censored_geometric(0.1, 20, false);
/// # geom.unwrap();
/// ```
pub fn sample_censored_geometric(prob: f64, max_trials: i64, enforce_constant_time: bool) -> Result<i64, &'static str> {

    // ensure that prob is a valid probability
    if prob < 0.0 || prob > 1.0 {return Err("probability is not within [0, 1]")}

    let mut bit: bool;
    let mut n_trials: i64 = 0;
    let mut geom_return: i64 = 0;

    // generate bits until we find a 1
    // if enforcing the runtime of the algorithm to be constant, the while loop
    // continues after the 1 is found and just stores the first location of a 1 bit.
    while n_trials < max_trials {
        bit = sample_bernoulli(prob, enforce_constant_time)?;
        n_trials += 1;

        // If we haven't seen a 1 yet, set the return to the current number of trials
        if bit && geom_return == 0 {
            geom_return = n_trials;
            if !enforce_constant_time {
                return Ok(geom_return);
            }
        }
    }

    // set geom_return to max if we never saw a bit equaling 1
    if geom_return == 0 {
        geom_return = max_trials; // could also set this equal to n_trials - 1.
    }

    Ok(geom_return)
}


#[cfg(feature = "use-mpfr")]
pub fn sample_laplace(sigma: f64) -> Result<f64, &'static str> {
    macro_rules! to_rug {($v:expr) => {rug::Float::with_val(53, $v)}}

    let rademacher_sample = if sample_standard_bernoulli()? {1.} else {-1.};
    let exponential_sample = {
        let mut rng = GeneratorOpenSSL {};
        let mut state = ThreadRandState::new_custom(&mut rng);
        let standard_exponential_sample = rug::Float::random_exp(&mut state);

        to_rug!(standard_exponential_sample) / to_rug!(sigma)
    };

    Ok(to_rug!(rademacher_sample * exponential_sample).to_f64())
}

#[cfg(not(feature = "use-mpfr"))]
pub fn sample_laplace(sigma: f64) -> Result<f64, &'static str> {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5, 0.5);
    Ok(u.signum() * (1.0 - 2.0 * u.abs()).ln() * sigma)
}


/// Generates a draw from a Gaussian(loc, scale) distribution using the MPFR library.
///
/// If shift = 0 and scale = 1, sampling is done in a way that respects exact rounding.
/// Otherwise, the return will be the result of a composition of two operations that
/// respect exact rounding (though the result will not necessarily).
///
/// # Arguments
/// * `shift` - The expectation of the Gaussian distribution.
/// * `scale` - The scaling parameter (standard deviation) of the Gaussian distribution.
///
/// # Return
/// Draw from Gaussian(loc, scale)
///
/// # Example
/// ```
/// use opendp::meas::sample_gaussian;
/// let gaussian = sample_gaussian(0.0, 1.0, false);
/// ```
#[cfg(feature = "use-mpfr")]
pub fn sample_gaussian(shift: f64, scale: f64, enforce_constant_time: bool) -> f64 {
    // mpfr is not compatible with constant-time queries
    assert!(!enforce_constant_time);

    // initialize 64-bit floats within mpfr/rug
    let mpfr_shift = Float::with_val(53, shift);
    let mpfr_scale = Float::with_val(53, scale);

    // initialize randomness
    let mut rng = GeneratorOpenSSL {};
    let mut state = ThreadRandState::new_custom(&mut rng);

    // generate Gaussian(0,1) according to mpfr standard, then convert to correct scale
    let gauss = Float::with_val(64, Float::random_normal(&mut state));
    gauss.mul_add(&mpfr_scale, &mpfr_shift).to_f64()

}

#[cfg(not(feature = "use-mpfr"))]
pub fn sample_gaussian(shift: f64, scale: f64, enforce_constant_time: bool) -> Result<f64, &'static str> {
    let uniform_sample = sample_standard_uniform(enforce_constant_time)?;
    Ok(shift + scale * std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * uniform_sample))
}
