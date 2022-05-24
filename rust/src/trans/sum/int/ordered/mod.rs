use num::Zero;

use crate::{
    core::{Function, StabilityMap, Transformation},
    dist::{AbsoluteDistance, InsertDeleteDistance, IntDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{AlertingAbs, CheckNull, DistanceConstant, InfCast, InfSub, SaturatingAdd},
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_int_ordered_sum<T>(
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<T>>,
        AllDomain<T>,
        InsertDeleteDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance> + CheckNull + Zero + AlertingAbs + SaturatingAdd + AddIsExact,
    IntDistance: InfCast<T>,
{
    let (lower, upper) = bounds.clone();
    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(lower.alerting_abs()?.total_max(upper)?),
    ))
}

pub fn make_sized_bounded_int_ordered_sum<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        InsertDeleteDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance> + InfSub + CheckNull + Zero + SaturatingAdd + AddIsExact,
    IntDistance: InfCast<T>,
{
    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2).and_then(|d_in| d_in.inf_mul(&range)),
        ),
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_bounded_int_ordered_sum() -> Fallible<()> {
        let trans = make_bounded_int_ordered_sum((1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        let trans = make_bounded_int_ordered_sum((1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        // test saturation arithmetic
        let trans = make_bounded_int_ordered_sum((1i8, 127))?;
        let sum = trans.invoke(&vec![-128, -128, 127, 127, 127])?;
        assert_eq!(sum, 127);

        Ok(())
    }

    #[test]
    fn test_make_sized_bounded_int_ordered_sum() -> Fallible<()> {
        let trans = make_sized_bounded_int_ordered_sum(4, (1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        let trans = make_sized_bounded_int_ordered_sum(4, (1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        Ok(())
    }
}
