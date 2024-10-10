use std::cmp::Ordering;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, IntDistance, SymmetricDistance},
    traits::{Integer, SaturatingAdd},
};

#[cfg(feature = "ffi")]
mod ffi;

#[doc(hidden)]
pub trait SplitSatSum: Sized {
    /// Method which takes an iterator and generates `Self` from the elements by
    /// "summing up" the items.
    #[allow(clippy::ptr_arg)]
    fn split_sat_sum(v: &Vec<Self>) -> Self;
}

macro_rules! impl_unsigned_int_split_sat_sum {
    ($($ty:ty)+) => ($(impl SplitSatSum for $ty {
        fn split_sat_sum(v: &Vec<Self>) -> Self {
            v.iter().fold(0, |sum, v| sum.saturating_add(*v))
        }
    })+);
}
macro_rules! impl___signed_int_split_sat_sum {
    ($($ty:ty)+) => ($(impl SplitSatSum for $ty {
        fn split_sat_sum(v: &Vec<Self>) -> Self {
            let (neg, pos) = v.iter().fold((0, 0), |(neg, pos), v| {
                match v.cmp(&0) {
                    Ordering::Less => (neg.saturating_add(&v), pos),
                    Ordering::Greater => (neg, pos.saturating_add(&v)),
                    Ordering::Equal => (neg, pos),
                }
            });
            neg.saturating_add(pos)
        }
    })+);
}

impl_unsigned_int_split_sat_sum! { u8 u16 u32 u64 u128 usize }
impl___signed_int_split_sat_sum! { i8 i16 i32 i64 i128 isize }

#[bootstrap(features("contrib"), generics(T(example = "$get_first(bounds)")))]
/// Make a Transformation that computes the sum of bounded ints.
/// Adds the saturating sum of the positives to the saturating sum of the negatives.
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
pub fn make_bounded_int_split_sum<T>(
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
    T: Integer + SplitSatSum,
{
    let (lower, upper) = bounds.clone();

    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?),
        AtomDomain::default(),
        Function::new(|arg: &Vec<T>| T::split_sat_sum(arg)),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(lower.alerting_abs()?.total_max(upper)?),
    )
}

#[bootstrap(features("contrib"), generics(T(example = "$get_first(bounds)")))]
/// Make a Transformation that computes the sum of bounded ints with known dataset size.
///
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
/// Adds the saturating sum of the positives to the saturating sum of the negatives.
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
pub fn make_sized_bounded_int_split_sum<T>(
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
    T: Integer + SplitSatSum,
{
    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;
    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?).with_size(size),
        AtomDomain::default(),
        Function::new(|arg: &Vec<T>| T::split_sat_sum(arg)),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2).and_then(|d_in| d_in.inf_mul(&range)),
        ),
    )
}

#[cfg(test)]
mod test;
