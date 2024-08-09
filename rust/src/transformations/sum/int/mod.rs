mod checked;
pub use checked::*;

mod monotonic;
pub use monotonic::*;

mod ordered;
pub use ordered::*;

mod split;
pub use split::*;

use crate::{
    error::Fallible,
    traits::{AlertingAbs, ExactIntCast, InfMul, ProductOrd},
};

#[doc(hidden)]
pub trait AddIsExact {}
macro_rules! impl_addition_is_exact {
    ($($ty:ty)+) => ($(impl AddIsExact for $ty {})+)
}
impl_addition_is_exact! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }

#[doc(hidden)]
pub trait CanIntSumOverflow: Sized {
    fn int_sum_can_overflow(size: usize, bounds: (Self, Self)) -> Fallible<bool>;
}

impl<T: ExactIntCast<usize> + AlertingAbs + ProductOrd + InfMul + AddIsExact> CanIntSumOverflow
    for T
{
    fn int_sum_can_overflow(size: usize, (lower, upper): (Self, Self)) -> Fallible<bool> {
        let size = T::exact_int_cast(size)?;
        let mag = lower.alerting_abs()?.total_max(upper)?;
        Ok(mag.inf_mul(&size).is_err())
    }
}
