#[cfg(feature="ffi")]
mod ffi;

use std::iter::Sum;
use std::ops::Sub;

use num::Zero;

use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::{AbsoluteDistance, IntDistance, SymmetricDistance};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{CheckNull, DistanceConstant, ExactIntCast, InfCast, InfDiv, SaturatingAdd, InfSub, AlertingAbs};

pub fn make_bounded_sum<T>(
    bounds: (T, T)
) -> Fallible<Transformation<VectorDomain<BoundedDomain<T>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>>
    where T: DistanceConstant<IntDistance> + Sub<Output=T> + SaturatingAdd + Zero + CheckNull + AlertingAbs,
          IntDistance: InfCast<T> {
    let (lower, upper) = bounds.clone();

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_constant(
            lower.alerting_abs()?.total_max(upper.alerting_abs()?)?)))
}

pub fn make_sized_bounded_sum<T>(
    size: usize, bounds: (T, T)
) -> Fallible<Transformation<SizedDomain<VectorDomain<BoundedDomain<T>>>, AllDomain<T>, SymmetricDistance, AbsoluteDistance<T>>> where
    T: DistanceConstant<IntDistance> + ExactIntCast<usize> + InfSub + CheckNull + InfDiv
    + Sub<Output=T>, for<'a> T: Sum<&'a T>,
    IntDistance: InfCast<T> {
    let size_ = T::exact_int_cast(size)?;
    let (lower, upper) = bounds.clone();

    lower.inf_mul(&size_).or(upper.inf_mul(&size_))
        .map_err(|_| err!(MakeTransformation, "potential for overflow when computing function"))?;

    let range = upper.inf_sub(&lower)?;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(
            BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityRelation::new_from_forward(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2)
                .and_then(|d_in| d_in.inf_mul(&range)))
    ))
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