mod checked;
use std::marker::PhantomData;

pub use checked::*;

mod ordered;
pub use ordered::*;

use num::{One, Zero};

use crate::{
    dist::IntDistance,
    error::Fallible,
    traits::{
        AlertingAbs, CheckNull, ExactIntCast, FloatBits, InfAdd, InfCast, InfDiv, InfLog2, InfMul,
        InfPow, InfSub, SaturatingAdd, TotalOrd,
    },
};

pub trait Float:
    CheckNull
    + num::Bounded
    + num::Float
    + Clone
    + TotalOrd
    + ExactIntCast<usize>
    + InfCast<usize>
    + InfCast<IntDistance>
    + InfAdd
    + InfMul
    + InfSub
    + InfDiv
    + InfPow
    + InfLog2
    + std::iter::Sum<Self>
    + FloatBits
    + ExactIntCast<Self::Bits>
    + Zero
    + One
    + AlertingAbs
    + SaturatingAdd
{
}
impl<T> Float for T where
    T: CheckNull
        + num::Bounded
        + num::Float
        + Clone
        + TotalOrd
        + ExactIntCast<usize>
        + InfCast<usize>
        + InfCast<IntDistance>
        + InfMul
        + InfSub
        + InfAdd
        + InfDiv
        + InfPow
        + InfLog2
        + std::iter::Sum<T>
        + ExactIntCast<T::Bits>
        + FloatBits
        + Zero
        + One
        + AlertingAbs
        + SaturatingAdd
{
}

// Marker type to represent sequential, or recursive summation
pub struct Sequential<T>(PhantomData<T>);

// Marker type to represent pairwise, or cascading summation
pub struct Pairwise<T>(PhantomData<T>);

pub trait SumRelaxation {
    type Item: Float;
    fn relaxation(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item>;
}

impl<T: Float> SumRelaxation for Sequential<T> {
    type Item = T;
    fn relaxation(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let size = T::exact_int_cast(size)?;
        let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
        let _2 = T::exact_int_cast(2)?;

        // n^2 / 2^(k - 1) max(|L|, U)
        size.inf_mul(&size)?
            .inf_div(&_2.inf_pow(&mantissa_bits.neg_inf_sub(&T::one())?)?)?
            .inf_mul(&lower.alerting_abs()?.total_max(upper)?)
    }
}

impl<T: Float> SumRelaxation for Pairwise<T> {
    type Item = T;
    fn relaxation(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let size = T::exact_int_cast(size)?;
        let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
        let _2 = T::exact_int_cast(2)?;

        // u * k where k = log_2(n)
        let uk = size
            .clone()
            .inf_log2()?
            .inf_div(&_2.inf_pow(&mantissa_bits)?)?;

        // 2uk/(1 - uk) n max(|L|, U)
        _2.inf_mul(&uk)?
            .inf_div(&T::one().neg_inf_sub(&uk)?)?
            .inf_mul(&size)?
            .inf_mul(&lower.alerting_abs()?.total_max(upper)?)
    }
}

pub trait CanFloatSumOverflow: SumRelaxation {
    fn float_sum_can_overflow(size: usize, bounds: (Self::Item, Self::Item)) -> Fallible<bool>;
}

impl<T: Float> CanFloatSumOverflow for Sequential<T> {
    fn float_sum_can_overflow(size: usize, (lower, upper): (T, T)) -> Fallible<bool> {
        let _2 = T::one() + T::one();
        let size_ = T::inf_cast(size)?;
        let mag = lower.alerting_abs()?.total_max(upper)?;

        // CHECK 1
        // If bound magnitude < ulp(T::MAX) / 2,
        //     then each addition to the accumulator will be unable to reach inf,
        //     because summations that reach the last band of floats will underflow/saturate.

        // ulp(T::MAX) / 2 = 2^(max_exponent - num_mantissa_bits) / 2
        // max_unbiased_exponent is always the same as the exponent bias
        let mag_limit = _2.powf(T::exact_int_cast(
            T::EXPONENT_BIAS - T::MANTISSA_BITS - T::Bits::one(),
        )?);
        if mag < mag_limit {
            // we can't overflow, because high magnitude additions will underflow
            return Ok(false);
        }

        // CHECK 2
        // The round up will never be by more than the next magnitude of 2
        // 2^ceil(log2(max(|L|, U))) * N is finite
        Ok(round_up_to_nearest_power_of_two(mag)?
            .inf_mul(&size_)
            .is_err())
    }
}

impl<T: Float> CanFloatSumOverflow for Pairwise<T> {
    fn float_sum_can_overflow(size: usize, (lower, upper): (T, T)) -> Fallible<bool> {
        let _2 = T::one() + T::one();
        let size_ = T::inf_cast(size)?;
        let mag = lower.alerting_abs()?.total_max(upper)?;

        // CHECK 1
        // If mag * N / 2 < ulp(T::MAX) / 2,
        //     then the final branch of the pairwise sum will underflow
        //     if the sum reaches the coarsest band of floats.
        // Therefore we want mag < ulp(T::MAX) / N

        // mag_limit = ulp(T::MAX) / N = 2^(max_unbiased_exponent - num_mantissa_bits)
        // max_unbiased_exponent is always the same as the exponent bias
        let max_ulp = _2.powf(T::exact_int_cast(T::EXPONENT_BIAS - T::MANTISSA_BITS)?);
        if mag < max_ulp.neg_inf_div(&size_)? {
            // we can't overflow, because the largest possible addition will underflow
            return Ok(false);
        }

        // CHECK 2
        // The round up will never be by more than the next magnitude of 2
        // 2^ceil(log2(max(|L|, U))) * N is finite
        Ok(round_up_to_nearest_power_of_two(mag)?
            .inf_mul(&size_)
            .is_err())
    }
}

pub fn round_up_to_nearest_power_of_two<T>(x: T) -> Fallible<T>
where
    T: ExactIntCast<T::Bits> + Float,
{
    if x.is_sign_negative() {
        return fallible!(
            FailedFunction,
            "get_smallest_greater_or_equal_power_of_two must have a positive argument"
        );
    }

    let exponent_bias = T::exact_int_cast(T::EXPONENT_BIAS)?;
    let exponent = T::exact_int_cast(x.exponent())?;
    // this subtraction is on small whole integers, so is exact
    let exponent_unbiased = exponent - exponent_bias;

    let pow = exponent_unbiased
        + if x.mantissa().is_zero() {
            T::zero()
        } else {
            T::one()
        };

    let _2 = T::one() + T::one();
    _2.inf_pow(&pow)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_up_to_nearest_power_of_two() -> Fallible<()> {
        assert_eq!(round_up_to_nearest_power_of_two(1.2)?, 2.);
        assert_eq!(round_up_to_nearest_power_of_two(2.0)?, 2.);
        assert_eq!(round_up_to_nearest_power_of_two(2.1)?, 4.);
        assert_eq!(
            round_up_to_nearest_power_of_two(1e23)?,
            151115727451828646838272.
        );
        assert_eq!(round_up_to_nearest_power_of_two(1e130)?, 11090678776483259438313656736572334813745748301503266300681918322458485231222502492159897624416558312389564843845614287315896631296.);

        Ok(())
    }

    #[test]
    fn test_float_sum_overflows_sequential() -> Fallible<()> {
        let almost_max = f64::from_bits(f64::MAX.to_bits() - 1);
        let ulp_max = f64::MAX - almost_max;
        let largest_size = usize::MAX;

        // should barely fail first check and significantly fail second check
        let can_of = Sequential::<f64>::float_sum_can_overflow(largest_size, (0., ulp_max / 2.))?;
        assert!(can_of);
        
        // should barely pass first check
        let can_of = Sequential::<f64>::float_sum_can_overflow(largest_size, (0., ulp_max / 4.))?;
        assert!(!can_of);

        // should barely fail first check and significantly pass second check
        let can_of = Sequential::<f64>::float_sum_can_overflow(10, (0., ulp_max / 2.))?;
        assert!(!can_of);
        Ok(())
    }
    
    #[test]
    fn test_float_sum_overflows_pairwise() -> Fallible<()> {
        let almost_max = f64::from_bits(f64::MAX.to_bits() - 1);
        let ulp_max = f64::MAX - almost_max;
        let largest_size = usize::MAX;

        // should fail both checks
        let can_of = Pairwise::<f64>::float_sum_can_overflow(largest_size, (0., ulp_max / 2.))?;
        assert!(can_of);

        // should barely fail first check and pass second check
        let can_of = Pairwise::<f64>::float_sum_can_overflow(largest_size, (0., ulp_max / (largest_size as f64)))?;
        assert!(!can_of);
        
        // should barely pass first check
        let can_of = Pairwise::<f64>::float_sum_can_overflow(largest_size, (0., ulp_max / (largest_size as f64) / 2.))?;
        assert!(!can_of);

        // should barely fail first check and significantly pass second check
        let can_of = Pairwise::<f64>::float_sum_can_overflow(10, (0., ulp_max / (largest_size as f64)))?;
        assert!(!can_of);
        Ok(())
    }
}
