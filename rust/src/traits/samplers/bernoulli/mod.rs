use std::ops::Neg;

use num::{One, Zero};

use crate::{
    error::Fallible,
    traits::{ExactIntCast, FloatBits, InfDiv},
};

use super::{fill_bytes, sample_geometric_buffer};

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
    /// use opendp::traits::samplers::SampleBernoulli;
    /// let n = bool::sample_bernoulli(0.7, false);
    /// # use opendp::error::ExplainUnwrap;
    /// # n.unwrap_test();
    /// ```
    /// ```should_panic
    /// // fails because 1.3 not a valid probability
    /// use opendp::traits::samplers::SampleBernoulli;
    /// let n = bool::sample_bernoulli(1.3, false);
    /// # use opendp::error::ExplainUnwrap;
    /// # n.unwrap_test();
    /// ```
    /// ```should_panic
    /// // fails because -0.3 is not a valid probability
    /// use opendp::traits::samplers::SampleBernoulli;
    /// let n = bool::sample_bernoulli(-0.3, false);
    /// # use opendp::error::ExplainUnwrap;
    /// # n.unwrap_test();
    /// ```
    fn sample_bernoulli(prob: T, constant_time: bool) -> Fallible<Self>;
}

impl<T> SampleBernoulli<T> for bool
where
    T: Copy + One + Zero + PartialOrd + FloatBits,
    T::Bits: PartialOrd + ExactIntCast<usize>,
    usize: ExactIntCast<T::Bits>,
{
    fn sample_bernoulli(prob: T, constant_time: bool) -> Fallible<Self> {
        // ensure that prob is a valid probability
        if !(T::zero()..=T::one()).contains(&prob) {
            return fallible!(FailedFunction, "probability is not within [0, 1]");
        }

        // if prob == 1., then exponent is T::EXPONENT_BIAS and mantissa is zero
        if prob.is_one() {
            return Ok(true);
        }

        // Consider the binary expansion of prob into an infinite sequence b_i
        //    prob = sum_{i=0}^\inf b_i / 2^(i + 1)
        // This algorithm samples i ~ Geometric(p=0.5), then returns b_i.

        // Step 1. sample first_heads_index = i ~ Geometric(p=0.5)
        let first_heads_index = {
            // Since prob has finite precision, there is some j for which b_i = 0 for all i > j.
            // Thus, it is equivalent to sample i from the truncated geometric, and return false if i > j.
            // j is the index of the last element of the binary expansion that could possibly be 1.
            //    j = max_coin_flips
            //      = max_{prob} [leading_zeros(prob) + mantissa_digits]
            //      = max_{prob} [max_prob_exponent - exponent(prob) + mantissa_digits]
            //      = max_{prob} [max_raw_prob_exponent - raw_exponent(prob) + mantissa_digits]
            //      = max_raw_exponent + mantissa_digits
            //               where max_raw_prob_exponent = T::EXPONENT_BIAS - 1 because prob < 1.
            //      = (T::EXPONENT_BIAS - 1) + (T::MANTISSA_BITS + 1)
            //      = T::EXPONENT_BIAS + T::MANTISSA_BITS
            let max_coin_flips =
                usize::exact_int_cast(T::EXPONENT_BIAS)? + usize::exact_int_cast(T::MANTISSA_BITS)?;

            // We need to sample at least j bits. The smallest sample size is a byte. Round up to the nearest byte:
            //    buffer_len = j.div_ceil(8)
            // When T = f64, we sample 135 bytes.
            //        = f32, we sample 19 bytes.
            // If the first heads is found after j flips, but before buffer_len * 8 flips,
            //    then it will always index into the trailing zeros of the binary expansion.
            let buffer_len = max_coin_flips.inf_div(&8)?;

            // repeatedly flip a fair coin (up to j times) to identify 0-based index i of first heads
            match sample_geometric_buffer(buffer_len, constant_time)? {
                // i is in terms of T::Bits, not usize; assign to first_heads_index
                Some(i) => T::Bits::exact_int_cast(i)?,
                // otherwise return early because i > j
                // i is beyond the greatest possible nonzero b_i
                None => return Ok(false),
            }
        };

        // Step 2. index into the binary expansion of prob at first_heads_index to get b_i

        // number of leading zeros in binary representation of prob
        //    exponent is bounded in [0, EXPONENT_BIAS - 1] by:
        //      1. check for valid probability
        //      2. and by returning when prob == 1
        let leading_zeros = T::EXPONENT_BIAS - T::Bits::one() - prob.raw_exponent();

        // if prob is >=.5, then leading_zeros = 0, and b_0 = 1, because the implicit bit is set.
        // if prob is .25,  then leading_zeros = 1, b_0 = 0, b_1 = 1, b_i = 0 for all i > 1
        // if prob is .125, then leading_zeros = 2, b_0 = 0, b_1 = 0, b_2 = 1, b_i = 0 for all i > 2
        // if prob is 0.3203125, then leading_zeros = 1, and only b_1, b_3, b_6 are set:
        //    b_1 + b_3 + b_6 = 2^-2 + 2^-4 + 2^-7 = 0.3203125

        Ok(match first_heads_index {
            // index into the leading zeros of the binary representation
            i if i < leading_zeros => false,
            // mantissa bit index -1 is implicitly set in ieee-754 when the exponent is nonzero
            i if i == leading_zeros => !prob.raw_exponent().is_zero(),
            // all other digits out-of-bounds are not float-approximated/are-implicitly-zero
            i if i > leading_zeros + T::MANTISSA_BITS => false,
            // retrieve the bit from the mantissa at `i` slots shifted from the left
            i => !(prob.to_bits() & T::Bits::one() << (leading_zeros + T::MANTISSA_BITS - i))
                .is_zero(),
        })
    }
}

pub trait SampleRademacher: Sized {
    fn sample_standard_rademacher() -> Fallible<Self>;
    fn sample_rademacher(prob: f64, constant_time: bool) -> Fallible<Self>;
}

impl<T: Neg<Output = T> + One> SampleRademacher for T {
    fn sample_standard_rademacher() -> Fallible<Self> {
        Ok(if bool::sample_standard_bernoulli()? {
            T::one()
        } else {
            T::one().neg()
        })
    }
    fn sample_rademacher(prob: f64, constant_time: bool) -> Fallible<Self> {
        Ok(if bool::sample_bernoulli(prob, constant_time)? {
            T::one()
        } else {
            T::one().neg()
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::samplers::test_utils::*;

    #[test]
    fn test_bernoulli() {
        [0.2, 0.5, 0.7, 0.9].iter().for_each(|p| {
            let sampler = || {
                if bool::sample_bernoulli(*p, false).unwrap() {
                    1.
                } else {
                    0.
                }
            };
            assert!(
                test_proportion_parameters(sampler, *p, 0.00001, *p / 100.),
                "empirical evaluation of the bernoulli({:?}) distribution failed",
                p
            )
        })
    }
}
