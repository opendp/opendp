use std::collections::Bound;
use std::iter::Sum;
use std::ops::Sub;

use num::Zero;

use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::{AbsoluteDistance, IntDistance, SymmetricDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{Abs, DistanceConstant, InfCast, SaturatingAdd, CheckedMul, ExactIntCast, CheckNull};

pub fn make_bounded_sum<T>(
    lower: T, upper: T
) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T> + Abs + SaturatingAdd + Zero + CheckNull,
          IntDistance: InfCast<T> {

    Ok(Transformation::new(
        VectorDomain::new(IntervalDomain::new(
            Bound::Included(lower.clone()), Bound::Included(upper.clone()))?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(lower.abs().total_max(upper.abs())?)))
}


pub fn make_bounded_sum_n<T>(
    lower: T, upper: T, length: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T>, for <'a> T: Sum<&'a T> + ExactIntCast<usize> + CheckedMul + CheckNull,
          IntDistance: InfCast<T> {
    let length_ = T::exact_int_cast(length)?;
    if lower.checked_mul(&length_).is_none()
        || upper.checked_mul(&length_).is_none() {
        return fallible!(MakeTransformation, "Detected potential for overflow when computing function.")
    }
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(IntervalDomain::new(
            Bound::Included(lower.clone()), Bound::Included(upper.clone()))?), length),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        // d_out >= d_in * (M - m) / 2
        StabilityRelation::new_from_constant((upper - lower) / T::exact_int_cast(2)?)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_bounded_sum_l1() {
        let transformation = make_bounded_sum::<i32>(0, 10).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_l2() {
        let transformation = make_bounded_sum::<i32>(0, 10).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_n() {
        let transformation = make_bounded_sum_n::<i32>(0, 10, 5).unwrap_test();
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.function.eval(&arg).unwrap_test();
        let expected = 15;
        assert_eq!(ret, expected);
    }
}