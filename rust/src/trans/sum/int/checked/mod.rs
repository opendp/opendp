use std::iter::Sum;

use crate::{
    core::{Function, StabilityMap, Transformation},
    dist::{AbsoluteDistance, IntDistance, SymmetricDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{CheckNull, DistanceConstant, InfCast, InfDiv, InfSub},
    trans::CanSumOverflow,
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_sized_bounded_int_checked_sum<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        SymmetricDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance> + InfSub + CheckNull + InfDiv + AddIsExact + CanSumOverflow,
    for<'a> T: Sum<&'a T>,
    IntDistance: InfCast<T>,
{
    if T::sum_can_overflow(size, bounds.clone()) {
        return fallible!(
            MakeTransformation,
            "potential for overflow when computing function"
        );
    }

    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
        SymmetricDistance::default(),
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
    fn test_make_sized_bounded_int_checked_sum() -> Fallible<()> {
        let trans = make_sized_bounded_int_checked_sum(4, (1, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        // should error under these conditions
        assert!(make_sized_bounded_int_checked_sum::<u8>(2, (0, 255)).is_err());
        Ok(())
    }
}
