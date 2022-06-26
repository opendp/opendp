#[cfg(feature = "ffi")]
mod ffi;

mod int;
pub use int::*;

mod float;
pub use float::*;

use crate::core::{Transformation, Metric};
use crate::dist::{AbsoluteDistance, SymmetricDistance, InsertDeleteDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{CheckNull, TotalOrd, InfMul, ExactIntCast};
use crate::trans::{make_ordered_random, make_unordered};

pub fn make_bounded_sum<MI, T>(bounds: (T, T)) -> Fallible<BoundedSumTrans<MI, T>>
    where MI: Metric, T: MakeBoundedSum<MI> {
    T::make_bounded_sum(bounds)
}
pub fn make_sized_bounded_sum<MI, T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<SizedBoundedSumTrans<MI, T>>
    where MI: Metric, T: MakeSizedBoundedSum<MI> {
    T::make_sized_bounded_sum(size, bounds)
}

// make_(sized_)?bounded_sum

// implementations delegate to:
// make_(sized_)?bounded_int_(checked|monotonic|ordered|split)_sum
// make_(sized_)?bounded_float_(checked|ordered)_sum

type BoundedSumTrans<MI, T> = Transformation<
    VectorDomain<BoundedDomain<T>>,
    AllDomain<T>,
    MI,
    AbsoluteDistance<T>,
>;

pub trait MakeBoundedSum<MI: Metric>: Sized + CheckNull + Clone + TotalOrd {
    fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<MI, Self>>;
}

macro_rules! impl_make_bounded_sum_int {
    ($($ty:ty)+) => {
        $(impl MakeBoundedSum<SymmetricDistance> for $ty {
            fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<SymmetricDistance, Self>> {
                // data size unknown, so checked sum is not applicable
                
                if Self::is_monotonic(bounds.clone()) {
                    // 1. if bounds share sign, then a simple saturating addition is associative
                    make_bounded_int_monotonic_sum(bounds)

                } else {
                    // 2. split sum is the cheapest remaining fallback when order is unknown
                    make_bounded_int_split_sum(bounds)
                }
            }
        })+
        $(impl MakeBoundedSum<InsertDeleteDistance> for $ty {
            fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<InsertDeleteDistance, Self>> {
                // data size unknown, so checked sum is not applicable

                // when input metric is ordered, ordered sum doesn't need a shuffle, making it comparatively cheaper
                // - ordered sum is more efficient than a split sum because splitting unnecessary
                // - ordered sum is tied with monotonic sum, but not conditional on monotonicity
                make_bounded_int_ordered_sum(bounds)
            }
        })+
    };
}
impl_make_bounded_sum_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }

const DEFAULT_SIZE_LIMIT: usize = 1_048_576; // 2^20
macro_rules! impl_make_bounded_sum_float {
    ($($ty:ty)+) => {
        $(impl MakeBoundedSum<SymmetricDistance> for $ty {
            fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<SymmetricDistance, Self>> {

                if !Self::sum_can_overflow(DEFAULT_SIZE_LIMIT, bounds) {
                    // 1. if the sum can't overflow, then use a more computationally efficient sum without saturation arithmetic
                    make_bounded_float_checked_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)

                } else {
                    // 2. sum can overflow, so shuffle and use an ordered sum
                    let domain = VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?);
                    make_ordered_random(domain)? >> make_bounded_float_ordered_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)?
                }
            }
        })+

        $(impl MakeBoundedSum<InsertDeleteDistance> for $ty {
            fn make_bounded_sum(bounds: (Self, Self)) -> Fallible<BoundedSumTrans<InsertDeleteDistance, Self>> {

                if !Self::sum_can_overflow(DEFAULT_SIZE_LIMIT, bounds) {
                    // 1. if the sum can't overflow, 
                    //    then do a no-op unordering and use a more computationally efficient sum without saturation arithmetic
                    let domain = VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?);
                    make_unordered(domain)? >> make_bounded_float_checked_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)?

                } else {
                    // 2. sum can overflow, but data is already sorted, so use a saturating sum
                    make_bounded_float_ordered_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)
                }
            }
        })+
    };
}
impl_make_bounded_sum_float! { f32 f64 }

type SizedBoundedSumTrans<MI, T> = Transformation<
    SizedDomain<VectorDomain<BoundedDomain<T>>>,
    AllDomain<T>,
    MI,
    AbsoluteDistance<T>,
>;
pub trait MakeSizedBoundedSum<MI: Metric>: Sized + CheckNull + Clone + TotalOrd {
    fn make_sized_bounded_sum(
        size: usize,
        bounds: (Self, Self),
    ) -> Fallible<SizedBoundedSumTrans<MI, Self>>;
}

macro_rules! impl_make_sized_bounded_sum_int {
    ($($ty:ty)+) => {
        $(impl MakeSizedBoundedSum<SymmetricDistance> for $ty {
            fn make_sized_bounded_sum(size: usize, bounds: (Self, Self)) -> Fallible<SizedBoundedSumTrans<SymmetricDistance, Self>> {

                if !Self::sum_can_overflow(size, bounds) {
                    // 1. if the sum can't overflow, don't need to worry about saturation arithmetic
                    make_sized_bounded_int_checked_sum(size, bounds)

                } else if Self::is_monotonic(bounds) {
                    // 2. a monotonic sum is less efficient due to saturation arithmetic
                    make_sized_bounded_int_monotonic_sum(size, bounds)

                } else {
                    // 3. a split sum is the least efficient, because it needs saturation arithmetic and separate pos/neg sums
                    make_sized_bounded_int_split_sum(size, bounds)
                }
            }
        })+

        $(impl MakeSizedBoundedSum<InsertDeleteDistance> for $ty {
            fn make_sized_bounded_sum(size: usize, bounds: (Self, Self)) -> Fallible<SizedBoundedSumTrans<InsertDeleteDistance, Self>> {

                if !Self::sum_can_overflow(size, bounds) {
                    // 1. if the sum can't overflow,
                    //    then do a no-op unordering and use a more computationally efficient sum without saturation arithmetic
                    let domain = SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?), size);
                    make_unordered(domain)? >> make_sized_bounded_int_checked_sum(size, bounds)?

                } else {
                    // when input metric is ordered, ordered sum doesn't need a shuffle, making it comparatively cheaper
                    // - ordered sum is more efficient than a split sum because splitting unnecessary
                    // - ordered sum is tied with monotonic sum, but not conditional on monotonicity
                    make_sized_bounded_int_ordered_sum(size, bounds)
                }
            }
        })+
    };
}
impl_make_sized_bounded_sum_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }
macro_rules! impl_make_sized_bounded_sum_float {
    ($($ty:ty)+) => {
        $(impl MakeSizedBoundedSum<SymmetricDistance> for $ty {
            fn make_sized_bounded_sum(size: usize, bounds: (Self, Self)) -> Fallible<SizedBoundedSumTrans<SymmetricDistance, Self>> {

                if !Self::sum_can_overflow(size, bounds) {
                    // 1. try the checked sum first, as floats are unlikely to overflow
                    make_sized_bounded_float_checked_sum::<Pairwise<_>>(size, bounds)    

                } else {
                    // 2. fall back to ordered summation
                    let domain = SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?), size);
                    make_ordered_random(domain)? >> make_sized_bounded_float_ordered_sum::<Pairwise<_>>(size, bounds)?
                }
            }
        })+

        $(impl MakeSizedBoundedSum<InsertDeleteDistance> for $ty {
            fn make_sized_bounded_sum(size: usize, bounds: (Self, Self)) -> Fallible<SizedBoundedSumTrans<InsertDeleteDistance, Self>> {

                if !Self::sum_can_overflow(size, bounds) {
                    // 1. if the sum can't overflow, 
                    //    then do a no-op unordering and use a more computationally efficient sum without saturation arithmetic
                    let domain = SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds.clone())?), size);
                    make_unordered(domain)? >> make_sized_bounded_float_checked_sum::<Pairwise<_>>(size, bounds)?

                } else {
                    // 2. fall back to ordered summation
                    make_sized_bounded_float_ordered_sum::<Pairwise<_>>(size, bounds)
                }
            }
        })+
    };
}
impl_make_sized_bounded_sum_float! { f32 f64 }

pub trait CanSumOverflow: Sized {
    fn sum_can_overflow(size: usize, bounds: (Self, Self)) -> bool;
}

impl<T: InfMul + ExactIntCast<usize>> CanSumOverflow for T {
    fn sum_can_overflow(size: usize, (lower, upper): (Self, Self)) -> bool {
        T::exact_int_cast(size)
            .and_then(|size| lower.inf_mul(&size).or(upper.inf_mul(&size)))
            .is_err()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_bounded_sum_l1() {
        let transformation = make_bounded_sum::<SymmetricDistance, i32>((0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = make_bounded_sum::<SymmetricDistance, i32>((0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_sized_bounded_sum() {
        let transformation = make_sized_bounded_sum::<SymmetricDistance, i32>(5, (0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }
}
