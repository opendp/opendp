use std::iter::Sum;
use std::ops::Sub;

use num::Zero;

use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::{AbsoluteDistance, IntDistance, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{Abs, DistanceConstant, InfCast, SaturatingAdd, CheckedMul, ExactIntCast, CheckNull};

pub fn make_bounded_sum<T>(
    bounds: (T, T)
) -> Fallible<Transformation<VectorDomain<BoundedDomain<T>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T> + Abs + SaturatingAdd + Zero + CheckNull,
          IntDistance: InfCast<T> {
    let (lower, upper) = bounds.clone();

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(lower.abs().total_max(upper.abs())?)))
}

// division with rounding towards infinity
pub trait InfDiv {
    fn inf_div(&self, other: &Self) -> Self;
}

macro_rules! impl_int_inf_div {
    ($($ty:ty),+) => ($(impl InfDiv for $ty {
        fn inf_div(&self, other: &Self) -> Self {
            (self + 1) / other
        }
    })+)
}
impl_int_inf_div!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_float_inf_div {
    ($($ty:ty),+) => ($(impl InfDiv for $ty {
        fn inf_div(&self, other: &Self) -> Self {
            let div = self / other;
            if !div.is_finite() {
                // don't increment -Inf or Inf into a NaN, leave NaN as-is
                div
            } else if div * other <= *self {
                // < is (probably) too tight, <= is too loose. Remain conservative with <=
                // perturb the floating-point bit representation by taking the next float
                <$ty>::from_bits(if div.is_sign_negative() {
                    div.to_bits() - 1
                } else {
                    div.to_bits() + 1
                })
            } else {
                div
            }
        }
    })+)
}
impl_float_inf_div!(f32, f64);

pub fn make_sized_bounded_sum<T>(
    size: usize, bounds: (T, T)
) -> Fallible<Transformation<SizedDomain<VectorDomain<BoundedDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T>, for <'a> T: Sum<&'a T> + ExactIntCast<usize> + CheckedMul + CheckNull + InfDiv,
          IntDistance: InfCast<T> {
    let size_ = T::exact_int_cast(size)?;
    let (lower, upper) = bounds.clone();
    if lower.checked_mul(&size_).is_none()
        || upper.checked_mul(&size_).is_none() {
        return fallible!(MakeTransformation, "Detected potential for overflow when computing function.")
    }
    let _2 = T::exact_int_cast(2)?;
    let range = upper - lower;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        // naively:
        // d_out >= d_in * (M - m) / 2
        // to avoid integer truncation:
        // d_out * 2 >= d_in * (M - m)
        StabilityRelation::new_all(
            enclose!((_2, range), move |&d_in: &IntDistance, d_out: &T|
                Ok(d_out.clone() * _2.clone() >= T::inf_cast(d_in)? * range.clone())),
            Some(move |d_in: &IntDistance| Ok(Box::new((T::inf_cast(*d_in)? * range.clone()).inf_div(&_2)))),
        None::<fn(&_) -> _>)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_bounded_sum_l1() {
        let transformation = make_bounded_sum::<i32>((0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = make_bounded_sum::<i32>((0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_n() {
        let transformation = make_sized_bounded_sum::<i32>(5, (0, 10)).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }
}