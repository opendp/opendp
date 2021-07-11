use std::cmp::Ordering;
use std::collections::Bound;
use std::iter::Sum;
use std::ops::Sub;

use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::{SymmetricDistance, AbsoluteDistance, IntDistance};
use crate::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{Abs, DistanceConstant, InfCast};

fn max<T: PartialOrd>(a: T, b: T) -> Option<T> {
    a.partial_cmp(&b).map(|o| if let Ordering::Less = o {b} else {a})
}

pub fn make_bounded_sum<T>(
    lower: T, upper: T
) -> Fallible<Transformation<VectorDomain<IntervalDomain<T>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T> + Abs,
          for <'a> T: Sum<&'a T>,
          IntDistance: InfCast<T> {

    Ok(Transformation::new(
        VectorDomain::new(IntervalDomain::new(
            Bound::Included(lower.clone()), Bound::Included(upper.clone()))?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(max(lower.abs(), upper.abs())
            .ok_or_else(|| err!(InvalidDistance, "lower and upper must be comparable"))?)))
}


pub fn make_bounded_sum_n<T>(
    lower: T, upper: T, length: usize
) -> Fallible<Transformation<SizedDomain<VectorDomain<IntervalDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T>, for <'a> T: Sum<&'a T>,
          IntDistance: InfCast<T> {

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(IntervalDomain::new(
            Bound::Included(lower.clone()), Bound::Included(upper.clone()))?), length),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        // d_out >= d_in * (M - m) / 2
        StabilityRelation::new_from_constant((upper - lower) / T::inf_cast(2)?)))
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