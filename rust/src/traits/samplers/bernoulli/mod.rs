use std::ops::Neg;

use num::{One, Zero};

use crate::error::Fallible;

use super::{fill_bytes, SampleExponent};

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



#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::samplers::test_utils::*;

    #[test]
    fn test_bernoulli() {
        [0.2, 0.5, 0.7, 0.9].iter().for_each(|p|
            assert!(test_proportion_parameters(
                || if bool::sample_bernoulli(*p, false).unwrap() {1.} else {0.},
                *p, 0.00001, *p / 100.),
                    "empirical evaluation of the bernoulli({:?}) distribution failed", p)
        )
    }
}