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
        InfPow, InfSub, SaturatingAdd, TotalOrd, SaturatingMul,
    },
};

use super::CanSumOverflow;

pub trait Float:
    CheckNull
    + num::Bounded
    + num::Float
    + Clone
    + TotalOrd
    + ExactIntCast<usize>
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
    + CanSumOverflow
    + SaturatingMul
{
}
impl<T> Float for T where
    T: CheckNull
        + num::Bounded
        + num::Float
        + Clone
        + TotalOrd
        + ExactIntCast<usize>
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
        + CanSumOverflow
        + SaturatingMul
{
}

// Marker type to represent sequential, or recursive summation
pub struct Sequential<T>(PhantomData<T>);

// Marker type to represent pairwise, or cascading summation
pub struct Pairwise<T>(PhantomData<T>);

pub trait SumRelaxation {
    type Item: Float;
    fn error(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item>;
    fn relaxation(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let _2 = Self::Item::exact_int_cast(2)?;
        _2.inf_mul(&Self::error(size, lower, upper)?)
    }
}

impl<T: Float> SumRelaxation for Sequential<T> {
    type Item = T;
    fn error(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let size = T::exact_int_cast(size)?;
        let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
        let _2 = T::exact_int_cast(2)?;

        // n^2 / 2^(k - 1) max(|L|, U)
        size.inf_mul(&size)?
            .inf_div(&_2.inf_pow(&mantissa_bits)?)?
            .inf_mul(&lower.alerting_abs()?.total_max(upper)?)
    }
}

impl<T: Float> SumRelaxation for Pairwise<T> {
    type Item = T;
    fn error(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let size = T::exact_int_cast(size)?;
        let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
        let _2 = T::exact_int_cast(2)?;

        // u * k where k = log_2(n)
        let uk = size
            .clone()
            .inf_log2()?
            .inf_div(&_2.inf_pow(&mantissa_bits)?)?;

        // (uk / (1 - uk)) n max(|L|, U)
        uk
            .inf_div(&T::one().neg_inf_sub(&uk)?)?
            .inf_mul(&size)?
            .inf_mul(&lower.alerting_abs()?.total_max(upper)?)
    }
}
