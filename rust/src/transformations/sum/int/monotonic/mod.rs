use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, IntDistance, SymmetricDistance},
    traits::Number,
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib"), generics(T(example = "$get_first(bounds)")))]
/// Make a Transformation that computes the sum of bounded ints,
/// where all values share the same sign.
///
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
///
/// # Arguments
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
///
/// # Generics
/// * `T` - Atomic Input Type and Output Type
pub fn make_bounded_int_monotonic_sum<T>(
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        AtomDomain<T>,
        SymmetricDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: Number + AddIsExact + IsMonotonic,
{
    if !T::is_monotonic(bounds.clone()) {
        return fallible!(
            MakeTransformation,
            "monotonic summation requires bounds to share the same sign"
        );
    }

    let (lower, upper) = bounds.clone();

    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?),
        AtomDomain::default(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(lower.alerting_abs()?.total_max(upper)?),
    )
}

#[bootstrap(features("contrib"), generics(T(example = "$get_first(bounds)")))]
/// Make a Transformation that computes the sum of bounded ints,
/// where all values share the same sign.
///
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
///
/// # Arguments
/// * `size` - Number of records in input data.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
///
/// # Generics
/// * `T` - Atomic Input Type and Output Type
pub fn make_sized_bounded_int_monotonic_sum<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        AtomDomain<T>,
        SymmetricDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: Number + AddIsExact + IsMonotonic,
{
    if !T::is_monotonic(bounds.clone()) {
        return fallible!(
            MakeTransformation,
            "monotonic summation requires bounds to share the same sign"
        );
    }

    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;

    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?).with_size(size),
        AtomDomain::default(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2).and_then(|d_in| d_in.inf_mul(&range)),
        ),
    )
}

#[doc(hidden)]
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
mod test;
