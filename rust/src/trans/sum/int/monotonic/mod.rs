use num::Zero;

use crate::{
    core::{Function, StabilityMap, Transformation},
    dist::{AbsoluteDistance, IntDistance, SymmetricDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{AlertingAbs, CheckNull, DistanceConstant, InfSub, SaturatingAdd},
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_bounded_int_monotonic_sum<T>(
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<T>>,
        AllDomain<T>,
        SymmetricDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: DistanceConstant<IntDistance>
        + CheckNull
        + Zero
        + AlertingAbs
        + SaturatingAdd
        + AddIsExact
        + IsMonotonic,
{
    if !T::is_monotonic(bounds.clone()) {
        return fallible!(
            MakeTransformation,
            "monotonic summation requires bounds to share the same sign"
        );
    }

    let (lower, upper) = bounds.clone();

    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(lower.alerting_abs()?.total_max(upper)?),
    ))
}

pub fn make_sized_bounded_int_monotonic_sum<T>(
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
    T: DistanceConstant<IntDistance>
        + InfSub
        + CheckNull
        + Zero
        + SaturatingAdd
        + AddIsExact
        + IsMonotonic,
{
    if !T::is_monotonic(bounds.clone()) {
        return fallible!(
            MakeTransformation,
            "monotonic summation requires bounds to share the same sign"
        );
    }

    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;

    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2).and_then(|d_in| d_in.inf_mul(&range)),
        ),
    ))
}

/// Checks if two elements of type T have the same sign
pub trait IsMonotonic: Sized {
    fn is_monotonic(bounds: (Self, Self)) -> bool;
}

macro_rules! impl_same_sign_signed_int {
    ($($ty:ty)+) => ($(impl IsMonotonic for $ty {
        fn is_monotonic((a, b): (Self, Self)) -> bool {
            a == 0 || b == 0 || (a > 0) == (b > 0)
        }
    })+)
}
impl_same_sign_signed_int! { i8 i16 i32 i64 i128 isize }

macro_rules! impl_same_sign_unsigned_int {
    ($($ty:ty)+) => ($(impl IsMonotonic for $ty {
        fn is_monotonic(_: (Self, Self)) -> bool {
            true
        }
    })+)
}
impl_same_sign_unsigned_int! { u8 u16 u32 u64 u128 usize }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_bounded_int_monotonic_sum() -> Fallible<()> {
        let trans = make_bounded_int_monotonic_sum((1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        let trans = make_bounded_int_monotonic_sum((1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        // should fail under these conditions
        assert!(make_bounded_int_monotonic_sum((-1i32, 1)).is_err());

        Ok(())
    }

    #[test]
    fn test_make_sized_bounded_int_monotonic_sum() -> Fallible<()> {
        let trans = make_sized_bounded_int_monotonic_sum(4, (1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        let trans = make_sized_bounded_int_monotonic_sum(4, (1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        Ok(())
    }
}
