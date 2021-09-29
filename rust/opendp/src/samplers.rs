use std::cmp;
use std::ops::{AddAssign, Neg, Sub, SubAssign};

use ieee754::Ieee754;
use num::{Bounded, clamp, One, Zero};
#[cfg(any(not(feature="use-mpfr"), not(feature="use-openssl")))]
use rand::Rng;
#[cfg(feature="use-mpfr")]
use rug::{Float, rand::{ThreadRandGen, ThreadRandState}};
#[cfg(not(feature="use-mpfr"))]
use statrs::function::erf;

use crate::error::Fallible;
#[cfg(any(not(feature="use-mpfr"), not(feature="use-openssl")))]
use rand::Rng;
use crate::traits::{TotalOrd, FloatBits};

#[cfg(feature="use-openssl")]
pub fn fill_bytes(buffer: &mut [u8]) -> Fallible<()> {
    use openssl::rand::rand_bytes;
    if let Err(e) = rand_bytes(buffer) {
        fallible!(FailedFunction, "OpenSSL error: {:?}", e)
    } else { Ok(()) }
}

#[cfg(not(feature="use-openssl"))]
pub fn fill_bytes(buffer: &mut [u8]) -> Fallible<()> {
    if let Err(e) = rand::thread_rng().try_fill(buffer) {
        fallible!(FailedFunction, "Rand error: {:?}", e)
    } else { Ok(()) }
}

#[cfg(feature = "use-mpfr")]
struct GeneratorOpenSSL {
    error: Fallible<()>,
}

impl GeneratorOpenSSL {
    fn new() -> Self {
        GeneratorOpenSSL { error: Ok(()) }
    }
}

#[cfg(feature="use-mpfr")]
impl ThreadRandGen for GeneratorOpenSSL {
    fn gen(&mut self) -> u32 {
        let mut buffer = [0u8; 4];
        if let Err(e) = fill_bytes(&mut buffer) {
            self.error = Err(e)
        }
        u32::from_ne_bytes(buffer)
    }
}

// SAMPLERS
pub trait SampleStandardBernoulli: Sized {
    fn sample_standard_bernoulli() -> Fallible<Self>;
}
impl SampleStandardBernoulli for bool {
    fn sample_standard_bernoulli() -> Fallible<bool> {
        let mut buffer = [0u8; 1];
        fill_bytes(&mut buffer)?;
        Ok(buffer[0] & 1 == 1)
    }
}

pub trait SampleBernoulli<T>: Sized {
    /// Sample a single bit with arbitrary probability of success
    ///
    /// Uses only an unbiased source of coin flips.
    /// The strategy for doing this with 2 flips in expectation is described [here](https://web.archive.org/web/20160418185834/https://amakelov.wordpress.com/2013/10/10/arbitrarily-biasing-a-coin-in-2-expected-tosses/).
    ///
    /// # Arguments
    /// * `prob`- The desired probability of success (bit = 1).
    /// * `constant_time` - Whether or not to enforce the algorithm to run in constant time
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
    fn sample_bernoulli(prob: T, constant_time: bool) -> Fallible<Self>;
}

impl<T: Copy + One + Zero + PartialOrd + SampleExponent> SampleBernoulli<T> for bool
    where T::Bits: PartialOrd {

    fn sample_bernoulli(prob: T, constant_time: bool) -> Fallible<Self> {

        // ensure that prob is a valid probability
        if !(T::zero()..=T::one()).contains(&prob) {
            return fallible!(FailedFunction, "probability is not within [0, 1]")
        }

        // repeatedly flip fair coin (up to 1023 times) and identify index (0-based) of first heads
        let first_heads_index = T::sample_exponent(constant_time)?;

        // if prob == 1., return after retrieving censored_specific_geom, to protect constant time
        // if prob == 1., then exponent is T::EXPONENT_PROB and mantissa is zero
        if prob == T::one() { return Ok(true) }

        // number of leading zeros in binary representation of prob
        //    cast is non-saturating because exponent only uses first 11 bits
        //    exponent is bounded in [0, EXPONENT_PROB] by check for valid probability and one check
        let num_leading_zeros = T::EXPONENT_PROB - prob.exponent();

        Ok(match first_heads_index {
            // index into the leading zeros of the binary representation
            i if i < num_leading_zeros => false,
            // bit index 0 is implicitly set in ieee-754 when the exponent is nonzero
            i if i == num_leading_zeros => prob.exponent() != T::Bits::zero(),
            // all other digits out-of-bounds are not float-approximated/are-implicitly-zero
            i if i > num_leading_zeros + T::MANTISSA_BITS => false,
            // retrieve the bit from the mantissa at `i` slots shifted from the left
            i => prob.to_bits() & (T::Bits::one() << (T::MANTISSA_BITS + num_leading_zeros - i)) != T::Bits::zero()
        })
    }
}

pub trait SampleRademacher: Sized {
    fn sample_standard_rademacher() -> Fallible<Self>;
    fn sample_rademacher(prob: f64, constant_time: bool) -> Fallible<Self>;
}

impl<T: Neg<Output=T> + One> SampleRademacher for T {
    fn sample_standard_rademacher() -> Fallible<Self> {
        Ok(if bool::sample_standard_bernoulli()? {T::one()} else {T::one().neg()})
    }
    fn sample_rademacher(prob: f64, constant_time: bool) -> Fallible<Self> {
        Ok(if bool::sample_bernoulli(prob, constant_time)? {T::one()} else {T::one().neg()})
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
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self>;
}

impl SampleUniform for f64 {
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self> {

        // A saturated mantissa with implicit bit is ~2
        let exponent: i16 = -(1 + f64::sample_exponent(constant_time)? as i16);

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
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self> {
        f64::sample_standard_uniform(constant_time).map(|v| v as f32)
    }
}

/// Return sample from a Geometric distribution with parameter p=0.5.
///
/// The algorithm generates B * 8 bits at random and returns
/// - Some(index of the first set bit)
/// - None (if all bits are 0)
///
/// This is a lower-level version of the sample_geometric trait
fn sample_geometric_buffer<const B: usize>(constant_time: bool) -> Fallible<Option<usize>> {
    Ok(if constant_time {
        let mut buffer = [0_u8; B];
        fill_bytes(&mut buffer)?;
        buffer.iter().enumerate()
            // ignore samples that contain no events
            .filter(|(_, &sample)| sample > 0)
            // compute the index of the smallest event in the batch
            .map(|(i, sample)| 8 * i + sample.leading_zeros() as usize)
            // retrieve the smallest index
            .min()

    } else {
        // retrieve up to B bytes, each containing 8 trials
        for i in 0..B {
            let mut buffer = vec![0_u8; 1];
            fill_bytes(&mut buffer)?;

            if buffer[0] > 0 {
                return Ok(Some(i * 8 + buffer[0].leading_zeros() as usize))
            }
        }
        None
    })
}

pub trait SampleExponent: FloatBits {
    fn sample_exponent(constant_time: bool) -> Fallible<Self::Bits>;
}
impl SampleExponent for f64 {
    fn sample_exponent(constant_time: bool) -> Fallible<Self::Bits> {
        // return index of the first true bit in a randomly sampled 128 byte buffer
        // return 1022 if no events occurred because 1023 is specially reserved for inf, -inf, NaN
        //     (incurs a slight violation of DP)
        let sample = sample_geometric_buffer::<128>(constant_time)?.unwrap_or(1022);
        Ok(cmp::min(sample, 1022) as Self::Bits)
    }
}
impl SampleExponent for f32 {
    fn sample_exponent(constant_time: bool) -> Fallible<Self::Bits> {
        // return index of the first true bit in a randomly sampled 16 byte buffer
        // return 126 if no events occurred because 127 is specially reserved for inf, -inf, NaN
        //     (incurs a slight violation of DP)
        let sample = sample_geometric_buffer::<16>(constant_time)?.unwrap_or(126);
        Ok(cmp::min(sample, 126) as Self::Bits)
    }
}


pub trait SampleGeometric: Sized {

    /// Sample from the censored geometric distribution with parameter `prob`.
    /// If `trials` is None, there are no timing protections, and the support is:
    ///     [Self::MIN, Self::MAX]
    /// If `trials` is Some, execution runs in constant time, and the support is
    ///     [Self::MIN, Self::MAX] ∩ {shift ±= {1, 2, 3, ..., `trials`}}
    ///
    /// Tail probabilities of the uncensored geometric accumulate at the extreme value of the support.
    ///
    /// # Arguments
    /// * `shift` - Parameter to shift the output by
    /// * `positive` - If true, positive noise is added, else negative
    /// * `prob` - Parameter for the geometric distribution, the probability of success on any given trial.
    /// * `trials` - If Some, run the algorithm in constant time with exactly this many trials.
    ///
    /// # Return
    /// A draw from the censored geometric distribution defined above.
    ///
    /// # Example
    /// ```
    /// use opendp::samplers::SampleGeometric;
    /// let geom = u8::sample_geometric(0, true, 0.1, Some(20));
    /// # use opendp::error::ExplainUnwrap;
    /// # geom.unwrap_test();
    /// ```
    fn sample_geometric(shift: Self, positive: bool, prob: f64, trials: Option<Self>) -> Fallible<Self>;
}

impl<T: Clone + Zero + One + PartialEq + AddAssign + SubAssign + Bounded> SampleGeometric for T {

    fn sample_geometric(mut shift: Self, positive: bool, prob: f64, mut trials: Option<Self>) -> Fallible<Self> {

        // ensure that prob is a valid probability
        if !(0.0..=1.0).contains(&prob) {return fallible!(FailedFunction, "probability is not within [0, 1]")}

        let bound = if positive { Self::max_value() } else { Self::min_value() };
        let mut success: bool = false;

        // loop must increment at least once
        loop {
            // make steps on `shift` until there is a successful trial or have reached the boundary
            if !success && shift != bound {
                if positive { shift += T::one() } else { shift -= T::one() }
            }

            // stopping criteria
            if let Some(trials) = trials.as_mut() {
                // in the constant-time regime, decrement trials until zero
                if trials.is_zero() { break }
                *trials -= T::one();
            } else if success {
                // otherwise break on first success
                break
            }

            // run a trial-- do we stop?
            success |= bool::sample_bernoulli(prob, trials.is_some())?;
        }
        Ok(shift)
    }
}

pub trait SampleTwoSidedGeometric: SampleGeometric {

    /// Sample from the censored two-sided geometric distribution with parameter `prob`.
    /// If `bounds` is None, there are no timing protections, and the support is:
    ///     [Self::MIN, Self::MAX]
    /// If `bounds` is Some, execution runs in constant time, and the support is
    ///     [Self::MIN, Self::MAX] ∩ {shift ±= {1, 2, 3, ..., `trials`}}
    ///
    /// Tail probabilities of the uncensored two-sided geometric accumulate at the extrema of the support.
    ///
    /// # Arguments
    /// * `shift` - Parameter to shift the output by
    /// * `scale` - Parameter to scale the output by
    /// * `bounds` - If Some, run the algorithm in constant time with both inputs and outputs clamped to this value.
    ///
    /// # Return
    /// A draw from the two-sided censored geometric distribution defined above.
    ///
    /// # Example
    /// ```
    /// use opendp::samplers::SampleTwoSidedGeometric;
    /// let geom = u8::sample_two_sided_geometric(0, 0.1, Some((20, 30)));
    /// # use opendp::error::ExplainUnwrap;
    /// # geom.unwrap_test();
    /// ```
    fn sample_two_sided_geometric(
        shift: Self, scale: f64, bounds: Option<(Self, Self)>
    ) -> Fallible<Self>;
}

impl<T: Clone + SampleGeometric + Sub<Output=T> + Bounded + Zero + One + TotalOrd> SampleTwoSidedGeometric for T {
    /// When no bounds are given, there are no protections against timing attacks.
    ///     The bounds are effectively T::MIN and T::MAX and up to T::MAX - T::MIN trials are taken.
    ///     The output of this mechanism is as if samples were taken from the
    ///         uncensored two-sided geometric distribution and saturated at the bounds of T.
    ///
    /// When bounds are given, samples are taken from the censored two-sided geometric distribution,
    ///     where the tail probabilities are accumulated in the +/- (upper - lower)th bucket from taking (upper - lower - 1) bernoulli trials.
    ///     This special bucket may at most appear at the clamping bound of the output distribution-
    ///     Should the shift be outside the bounds, this irregular bucket and its zero-neighbor bucket would both be present in the output.
    ///     There is no multiplicative bound on the difference in probabilities between the output probabilities for neighboring datasets.
    ///     Therefore the input must be clamped. In addition, the noised output must be clamped as well--
    ///         if the greatest magnitude noise GMN = (upper - lower), then should (upper + GMN) be released,
    ///             the analyst can deduce that the input was greater than or equal to upper
    fn sample_two_sided_geometric(mut shift: T, scale: f64, bounds: Option<(Self, Self)>) -> Fallible<Self>  {
        let trials: Option<T> = if let Some((lower, upper)) = bounds.clone() {
            // if the output interval is a point
            if lower == upper {return Ok(lower)}
            Some(upper - lower - T::one())
        } else {None};

        let alpha: f64 = (scale.recip()).exp().recip();

        // It should be possible to drop the input clamp at a cost of `delta = 2^(-(upper - lower))`.
        // Thanks for the input @ctcovington (Christian Covington)
        if let Some((lower, upper)) = &bounds {
            shift = clamp(shift, lower.clone(), upper.clone());
        }

        // TODO: check MIR for reordering that moves these samples inside the conditional
        // TODO: benchmark execution time on different inputs
        let uniform = f64::sample_standard_uniform(bounds.is_some())?;
        let direction = bool::sample_standard_bernoulli()?;
        let geometric = T::sample_geometric(shift.clone(), direction,1. - alpha, trials)?;

        // add 0 noise with probability (1-alpha) / (1+alpha), otherwise use geometric sample
        let noised = if uniform < (1. - alpha) / (1. + alpha) { shift } else { geometric };

        Ok(if let Some((lower, upper)) = bounds {
            clamp(noised, lower, upper)
        } else {
            noised
        })
    }
}


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


pub trait MantissaDigits { const MANTISSA_DIGITS: u32; }

impl MantissaDigits for f32 { const MANTISSA_DIGITS: u32 = f32::MANTISSA_DIGITS; }

impl MantissaDigits for f64 { const MANTISSA_DIGITS: u32 = f64::MANTISSA_DIGITS; }

#[cfg(feature = "use-mpfr")]
pub trait CastInternalReal: MantissaDigits + Sized {
    fn from_internal(v: Float) -> Self;
    fn into_internal(self) -> Float;
}

#[cfg(not(feature = "use-mpfr"))]
pub trait CastInternalReal: rand::distributions::uniform::SampleUniform + SampleGaussian {
    fn from_internal(v: Self) -> Self;
    fn into_internal(self) -> Self;
}

#[cfg(feature = "use-mpfr")]
impl CastInternalReal for f64 {
    fn from_internal(v: Float) -> Self { v.to_f64() }
    fn into_internal(self) -> Float { rug::Float::with_val(Self::MANTISSA_DIGITS, self) }
}

#[cfg(feature = "use-mpfr")]
impl CastInternalReal for f32 {
    fn from_internal(v: Float) -> Self { v.to_f32() }
    fn into_internal(self) -> Float { rug::Float::with_val(Self::MANTISSA_DIGITS, self) }
}

#[cfg(not(feature = "use-mpfr"))]
impl CastInternalReal for f64 {
    fn from_internal(v: f64) -> Self { v }
    fn into_internal(self) -> Self { self }
}

#[cfg(not(feature = "use-mpfr"))]
impl CastInternalReal for f32 {
    fn from_internal(v: f32) -> Self { v }
    fn into_internal(self) -> Self { self }
}

#[cfg(feature = "use-mpfr")]
impl<T: CastInternalReal + SampleRademacher + Zero> SampleLaplace for T {
    fn sample_laplace(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        if scale.is_zero() { return Ok(shift) }
        if constant_time {
            return fallible!(FailedFunction, "mpfr samplers do not support constant time execution")
        }

        // initialize randomness
        let mut rng = GeneratorOpenSSL::new();
        let laplace = {
            let mut state = ThreadRandState::new_custom(&mut rng);

            // see https://arxiv.org/pdf/1303.6257.pdf, algorithm V for exact standard exponential deviates
            let exponential = rug::Float::with_val(
                Self::MANTISSA_DIGITS, rug::Float::random_exp(&mut state));
            // adding a random sign to the exponential deviate does not induce gaps or stacks
            exponential * T::sample_standard_rademacher()?.into_internal()
        };
        rng.error?;

        use rug::float::Round;
        use rug::ops::{DivAssignRound, MulAssignRound};
        let scale = scale.into_internal();

        // (shift / scale + noise) * scale.
        let mut value = shift.into_internal();
        // when scaling into the noise coordinate space, round down so that noise is overestimated
        value.div_assign_round(&scale, Round::Down);
        // the noise itself is never scaled
        value.add_assign(&laplace);
        // postprocessing remains differentially private
        value.mul_assign_round(&scale, Round::Up);

        Ok(Self::from_internal(value))
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
        Ok(shift + u.signum() * u.abs().ln() * scale)
    }
}

#[cfg(feature = "use-mpfr")]
impl<T: CastInternalReal + Zero> SampleGaussian for T {

    fn sample_gaussian(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        if scale.is_zero() { return Ok(shift) }
        if constant_time {
            return fallible!(FailedFunction, "mpfr samplers do not support constant time execution")
        }

        // initialize randomness
        let mut rng = GeneratorOpenSSL::new();
        let gauss = {
            let mut state = ThreadRandState::new_custom(&mut rng);

            // generate Gaussian(0,1) according to mpfr standard
            // See https://arxiv.org/pdf/1303.6257.pdf, algorithm N for exact standard normal deviates
            rug::Float::with_val(Self::MANTISSA_DIGITS, Float::random_normal(&mut state))
        };
        rng.error?;

        use rug::float::Round;
        use rug::ops::{DivAssignRound, MulAssignRound};
        let scale = scale.into_internal();

        // (shift / scale + noise) * scale.
        let mut value = shift.into_internal();
        // when scaling into the noise coordinate space, round down so that noise is overestimated
        value.div_assign_round(&scale, Round::Down);
        // the noise itself is never scaled
        value.add_assign(&gauss);
        // postprocessing remains differentially private
        value.mul_assign_round(&scale, Round::Up);

        Ok(Self::from_internal(value))
    }
}


#[cfg(not(feature = "use-mpfr"))]
impl SampleGaussian for f64 {
    fn sample_gaussian(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        let uniform_sample = f64::sample_standard_uniform(constant_time)?;
        Ok(shift + scale * std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * uniform_sample))
    }
}

#[cfg(not(feature = "use-mpfr"))]
impl SampleGaussian for f32 {
    fn sample_gaussian(shift: Self, scale: Self, constant_time: bool) -> Fallible<Self> {
        let uniform_sample = f64::sample_standard_uniform(constant_time)?;
        Ok(shift + scale * std::f32::consts::SQRT_2 * (erf::erfc_inv(2.0 * uniform_sample) as f32))
    }
}

pub trait SampleUniformInt: Sized {
    /// sample uniformly from [Self::MIN, Self::MAX]
    fn sample_uniform_int() -> Fallible<Self>;
    /// sample uniformly from [0, upper)
    fn sample_uniform_int_0_u(upper: Self) -> Fallible<Self>;
}

macro_rules! impl_sample_uniform_unsigned_int {
    ($($ty:ty),+) => ($(
        impl SampleUniformInt for $ty {
            fn sample_uniform_int() -> Fallible<Self> {
                let mut buffer = [0; core::mem::size_of::<Self>()];
                fill_bytes(&mut buffer).unwrap();
                Ok(Self::from_be_bytes(buffer))
            }
            fn sample_uniform_int_0_u(upper: Self) -> Fallible<Self> {
                // v % upper is unbiased for any v < MAX - MAX % upper, because
                // MAX - MAX % upper evenly folds into [0, upper) RAND_MAX/upper times
                loop {
                    // algorithm is only valid when sample_uniform_int is non-negative
                    let v = Self::sample_uniform_int()?;
                    if v <= Self::MAX - Self::MAX % upper {
                        return Ok(v % upper)
                    }
                }
            }
        }
    )+)
}
impl_sample_uniform_unsigned_int!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    #[ignore]
    fn test_sample_uniform_int() -> Fallible<()> {
        let mut counts = HashMap::new();
        // this checks that the output distribution of each number is uniform
        (0..10000).try_for_each(|_| {
            let sample = u32::sample_uniform_int_0_u(7)?;
            *counts.entry(sample).or_insert(0) += 1;
            Fallible::Ok(())
        })?;
        println!("{:?}", counts);
        Ok(())
    }
}