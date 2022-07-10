use crate::{error::Fallible, traits::{FloatBits, ExactIntCast}};

use super::{fill_bytes, sample_geometric_buffer};
use ieee754::Ieee754;

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
    /// use opendp::traits::samplers::SampleUniform;
    /// let unif = f64::sample_standard_uniform(false);
    /// # use opendp::error::ExplainUnwrap;
    /// # unif.unwrap_test();
    /// ```
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self>;
}

impl SampleUniform for f64 {
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self> {
        // The unbiased exponent of Uniform([0, 1)) is in [-1023, -1].
        //
        // # Lower bound of -1023:
        // Zero and subnormal numbers have a biased exponent of 0 -> an unbiased exponent of -1023
        // 
        // # Upper bound of -1:
        // A saturated mantissa is ~2, so the unbiased exponent must be <= -1, because Uniform([0, 1)) is < 1.
        //   sign     exp    mantissa
        //   (-1)^0 * 2^-1 * 1.9999... ~ 1

        // Use rejection sampling to draw ~ TruncatedGeometric(p=0.5, bounds=[0, 1022])
        // Reject samples > 1022 to redistribute the probability amongst all exponent bands
        let truncated_geometric_sample = loop {
            // find index of the first true bit in a randomly sampled byte buffer
            let exponent = sample_geometric_buffer(Self::EXPONENT_UNIFORM_LEN, constant_time)?
                // cast to the bits type. This cast lossless and infallible
                .map(|v| v as u64)
                // reject success on last coin flip because last flip is reserved for inf, -inf, NaN
                .and_then(|v| (v != Self::EXPONENT_BIAS).then_some(v));

            if let Some(e) = exponent {
                break e
            }
        };

        // Transform [0, 1022] -> [-1023, -1]
        let exponent = -i16::exact_int_cast(truncated_geometric_sample)? - 1;

        let mantissa: u64 = {
            // Of a 64 bit buffer, we want the first 12 bits to be zero, 
            //    and the last 52 bits to be uniformly random
            let mut mantissa_buffer = [0u8; 8];
            // Fill the last 56 bits with randomness.
            fill_bytes(&mut mantissa_buffer[1..])?;
            // Clear the leftmost four bits of the second byte
            mantissa_buffer[1] &= 0b00001111;

            // convert mantissa to integer
            u64::from_be_bytes(mantissa_buffer)
        };

        // Generate uniform random number from [0,1)
        Ok(Self::recompose(false, exponent, mantissa))
    }
}

impl SampleUniform for f32 {
    fn sample_standard_uniform(constant_time: bool) -> Fallible<Self> {
        // The unbiased exponent of Uniform([0, 1)) is in [-127, -1].
        //
        // # Lower bound of -127:
        // Zero and subnormal numbers have a biased exponent of 0 -> an unbiased exponent of -127
        // 
        // # Upper bound of -1:
        // A saturated mantissa is ~2, so the unbiased exponent must be <= -1, because Uniform([0, 1)) is < 1.
        //   sign     exp    mantissa
        //   (-1)^0 * 2^-1 * 1.9999... ~ 1

        // Use rejection sampling to draw ~ TruncatedGeometric(p=0.5, bounds=[0, 126])
        // Reject samples > 126 to redistribute the probability amongst all exponent bands
        let truncated_geometric_sample = loop {

            // find index of the first true bit in a randomly sampled byte buffer
            let exponent = sample_geometric_buffer(Self::EXPONENT_UNIFORM_LEN, constant_time)?
                // cast to the bits type. This cast lossless and infallible
                .map(|v| v as u32)
                // reject success on last coin flip because last flip is reserved for inf, -inf, NaN
                .and_then(|v| (v != Self::EXPONENT_BIAS).then_some(v));
                

            if let Some(e) = exponent {
                break e
            }
        };

        // Transform [0, 126] -> [-127, -1]
        let exponent = -i16::exact_int_cast(truncated_geometric_sample)? - 1;

        let mantissa: u32 = {
            // Of a 32 bit buffer, we want the first 9 bits to be zero, 
            //    and the last 23 bits to be uniformly random
            let mut mantissa_buffer = [0u8; 4];
            // Fill the last 24 bits with randomness.
            fill_bytes(&mut mantissa_buffer[1..])?;
            // Clear the leftmost bit of the second byte
            mantissa_buffer[1] &= 0b01111111;

            // convert mantissa to integer
            u32::from_be_bytes(mantissa_buffer)
        };

        // Generate uniform random number from [0,1)
        Ok(Self::recompose(false, exponent, mantissa))
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
                fill_bytes(&mut buffer)?;
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
mod test_uniform_int {
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