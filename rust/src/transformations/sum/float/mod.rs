mod checked;
use std::marker::PhantomData;

pub use checked::*;

mod ordered;
pub use ordered::*;

use crate::{
    error::Fallible,
    traits::{Float, InfAdd},
};

/// Marker type to represent sequential, or recursive summation
pub struct Sequential<T>(PhantomData<T>);

/// Marker type to represent pairwise, or cascading summation
pub struct Pairwise<T>(PhantomData<T>);

#[doc(hidden)]
pub trait SumRelaxation {
    type Item: Float;
    fn error(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item>;
    fn relaxation(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let error = Self::error(size, lower, upper)?;
        error.inf_add(&error)
    }
}

impl<T: Float> SumRelaxation for Sequential<T> {
    type Item = T;
    fn error(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let size = T::exact_int_cast(size)?;
        let _2 = T::exact_int_cast(2)?;

        // n^2 / 2^(k - 1) max(|L|, U)
        size.inf_mul(&size)?
            .inf_div(&_2.inf_powi(T::MANTISSA_BITS.into())?)?
            .inf_mul(&lower.alerting_abs()?.total_max(upper)?)
    }
}

impl<T: Float> SumRelaxation for Pairwise<T> {
    type Item = T;
    fn error(size: usize, lower: Self::Item, upper: Self::Item) -> Fallible<Self::Item> {
        let size = T::exact_int_cast(size)?;
        let _2 = T::exact_int_cast(2)?;
        // u * k where k = log_2(n)
        let uk = size
            .inf_log2()?
            .inf_div(&_2.inf_powi(T::MANTISSA_BITS.into())?)?;
        // (uk / (1 - uk)) n max(|L|, U)
        uk.inf_div(&T::one().neg_inf_sub(&uk)?)?
            .inf_mul(&size)?
            .inf_mul(&lower.alerting_abs()?.total_max(upper)?)
    }
}
