use std::{mem::size_of, ops::Sub};

use crate::{
    error::Fallible,
    traits::{ExactIntCast, FloatBits, InfDiv},
};

use super::{fill_bytes, sample_geometric_buffer};

use dashu::{
    base::{BitTest, Signed},
    integer::{IBig, UBig},
};
use num::{Integer, One};

/// Sample exactly from the uniform distribution.
pub trait SampleUniform: Sized {
    /// # Proof Definition
    /// Return `Err(e)` if there is insufficient system entropy, or
    /// `Ok(sample)`, where `sample` is a draw from Uniform[0,1).
    ///
    /// For non-uniform data types like floats,
    /// the probability of sampling each value is proportional to the distance to the next neighboring float.
    ///
    /// # Example
    /// ```
    /// // valid draw from Unif[0,1)
    /// use opendp::traits::samplers::SampleUniform;
    /// let unif = f64::sample_standard_uniform(false);
    /// # use opendp::error::ExplainUnwrap;
    /// # unif.unwrap_test();
    /// ```
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self>;
}

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
impl<T, B> SampleUniform for T
where
    T: SampleMantissa<Bits = B>,
    B: ExactIntCast<usize> + Sub<Output = B> + One,
    usize: ExactIntCast<B>,
{
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self> {
        // The unbiased exponent of Uniform([0, 1)) is in
        //   f64: [-1023, -1]; f32: [-127, -1]
        //
        // # Lower bound:
        // Zero and subnormal numbers have a biased exponent of 0 -> an unbiased exponent of -1023 or -127
        //
        // # Upper bound of -1:
        // A saturated mantissa is ~2, so the unbiased exponent must be <= -1, because Uniform([0, 1)) is < 1.
        //   sign     exp    mantissa
        //   (-1)^0 * 2^-1 * 1.9999... ~ 1

        let max_coin_flips = usize::exact_int_cast(T::EXPONENT_BIAS)? - 1;

        // round up to the next number of bytes. 128 for f64, 16 for f32
        let buffer_len = max_coin_flips.inf_div(&8)?;

        // Use rejection sampling to draw ~ TruncatedGeometric(p=0.5, bounds=[0, buffer_len * 8])
        // Reject samples > max_coin_flips to redistribute the probability amongst all exponent bands
        let truncated_geometric_sample = loop {
            // find index of the first true bit in a randomly sampled byte buffer
            let sample = sample_geometric_buffer(buffer_len, constant_time)?
                // reject success on extra trailing bits of last byte in the buffer
                .and_then(|v| (v < max_coin_flips).then(|| v));

            if let Some(e) = sample {
                // cast to the bits type. This cast is lossless and infallible
                break B::exact_int_cast(e)?;
            }
        };

        let raw_exponent = T::EXPONENT_BIAS - B::one() - truncated_geometric_sample;
        let mantissa = T::sample_mantissa()?;

        // Generate uniform random number from [0,1)
        Ok(Self::from_raw_components(false, raw_exponent, mantissa))
    }
}

/// Sample the mantissa of a uniformly-distributed floating-point number.
trait SampleMantissa: FloatBits {
    /// # Proof Definition
    /// Returns `Err(e)` if there is insufficient system entropy, or
    /// `Some(sample)`, where `sample` is a bit-vector of zeros,
    /// but the last Self::MANTISSA_BITS are iid Bernoulli(p=0.5) draws.
    fn sample_mantissa() -> Fallible<Self::Bits>;
}

macro_rules! impl_sample_mantissa {
    ($ty:ty, $mask:literal) => {
        impl SampleMantissa for $ty {
            fn sample_mantissa() -> Fallible<Self::Bits> {
                // Of a 64 or 32 bit buffer, we want the first 12 or 9 bits to be zero,
                //    and the last 52 or 23 bits to be uniformly random
                let mut mantissa_buffer = [0u8; size_of::<Self>()];
                // Fill the last 56 or 24 bits with randomness.
                fill_bytes(&mut mantissa_buffer[1..])?;
                // Clear the leftmost 4 or 1 bits of the second byte
                mantissa_buffer[1] &= $mask;

                // convert buffer to integer bits
                Ok(Self::Bits::from_be_bytes(mantissa_buffer))
            }
        }
    };
}

impl_sample_mantissa!(f64, 0b00001111);
impl_sample_mantissa!(f32, 0b01111111);

/// Sample an integer uniformly over `[Self::MIN, Self::MAX]`.
pub trait SampleUniformInt: Sized {
    /// # Proof Definition
    /// Return either `Err(e)` if there is insufficient system entropy,
    /// or `Some(sample)`, where `sample` is uniformly distributed over `[Self::MIN, Self::MAX]`.
    fn sample_uniform_int() -> Fallible<Self>;
}

/// Sample an integer uniformly over `[Self::MIN, upper)`
pub trait SampleUniformIntBelow: Sized {
    /// # Proof Definition
    /// For any setting of `upper`,
    /// return either `Err(e)` if there is insufficient system entropy,
    /// or `Some(sample)`, where `sample` is uniformly distributed over `[Self::MIN, upper)`.
    fn sample_uniform_int_below(upper: Self, trials: Option<usize>) -> Fallible<Self>;
}

macro_rules! impl_sample_uniform_unsigned_int {
    ($($ty:ty),+) => ($(
        impl SampleUniformInt for $ty {
            fn sample_uniform_int() -> Fallible<Self> {
                let mut buffer = [0; core::mem::size_of::<Self>()];
                fill_bytes(&mut buffer)?;
                Ok(Self::from_be_bytes(buffer))
            }
        }
        impl SampleUniformIntBelow for $ty {
            fn sample_uniform_int_below(upper: Self, mut trials: Option<usize>) -> Fallible<Self> {
                let mut found = None;
                let threshold = Self::MAX - Self::MAX % upper;

                loop {
                    if trials == Some(0) {
                        return found.ok_or_else(|| {
                            err!(
                                FailedFunction,
                                "failed to sample a number within the allotted number of trials"
                            )
                        });
                    }
                    trials.as_mut().map(|t| *t -= 1);

                    // algorithm is only valid when sample_uniform_int is non-negative
                    let sample = Self::sample_uniform_int()?;
                    if sample < threshold && found.is_none() {
                        found = Some(sample % &upper);
                    }

                    if found.is_some() && trials.is_none() {
                        // v % upper is unbiased for any v < MAX - MAX % upper, because
                        // MAX - MAX % upper evenly folds into [0, upper) RAND_MAX/upper times
                        return Ok(found.unwrap());
                    }
                }
            }
        }
    )+)
}
impl_sample_uniform_unsigned_int!(u8, u16, u32, u64, u128, usize);

impl SampleUniformIntBelow for UBig {
    fn sample_uniform_int_below(upper: Self, mut trials: Option<usize>) -> Fallible<Self> {
        // ceil(ceil(log_2(upper)) / 8)
        let byte_len = Integer::div_ceil(&upper.bit_len(), &8);

        // sample % upper is unbiased for any sample < threshold, because
        // max - max % upper evenly folds into [0, upper) max/upper times
        let max = UBig::from_be_bytes(&vec![u8::MAX; byte_len]);
        let threshold = &max - &max % &upper;

        let mut buffer = vec![0; byte_len];
        let mut found = None;

        loop {
            if trials == Some(0) {
                return found.ok_or_else(|| {
                    err!(
                        FailedFunction,
                        "failed to sample a number within the allotted number of trials"
                    )
                });
            }
            trials.as_mut().map(|t| *t -= 1);

            fill_bytes(&mut buffer)?;

            let sample = UBig::from_be_bytes(&buffer);
            if sample < threshold && found.is_none() {
                found = Some(sample % &upper);
            }

            if found.is_some() && trials.is_none() {
                return Ok(found.unwrap());
            }
        }
    }
}

impl SampleUniformIntBelow for IBig {
    fn sample_uniform_int_below(upper: Self, trials: Option<usize>) -> Fallible<Self> {
        if upper.is_negative() {
            return fallible!(
                FailedFunction,
                "upper bound ({}) must not be negative",
                upper
            );
        }

        let upper = UBig::try_from(upper)?;
        let sample = UBig::sample_uniform_int_below(upper, trials)?;
        Ok(sample.into())
    }
}

#[cfg(test)]
mod test;
