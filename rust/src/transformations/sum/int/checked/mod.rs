use std::iter::Sum;

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, IntDistance, SymmetricDistance},
    traits::Number,
    transformations::CanIntSumOverflow,
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib"), generics(T(example = "$get_first(bounds)")))]
/// Make a Transformation that computes the sum of bounded ints.
/// The effective range is reduced, as (bounds * size) must not overflow.
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
pub fn make_sized_bounded_int_checked_sum<T>(
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
    T: Number + AddIsExact + CanIntSumOverflow,
    for<'a> T: Sum<&'a T>,
{
    if T::int_sum_can_overflow(size, bounds.clone())? {
        return fallible!(
            MakeTransformation,
            "potential for overflow when computing function"
        );
    }

    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;
    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?).with_size(size),
        AtomDomain::default(),
        Function::new(|arg: &Vec<T>| arg.iter().sum()),
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
