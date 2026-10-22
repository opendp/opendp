//! Ordered float sums use exact sequential saturation as their reference function.
//! Clamping is monotone and nonexpansive, so aligning neighboring inputs keeps
//! common suffixes from amplifying edit differences. Sequential::relaxation
//! additionally accounts for rounding on each input. Regrouping saturated partial
//! sums does not have this property, so Pairwise is intentionally unsupported.

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, InsertDeleteDistance, IntDistance},
    traits::{AlertingAbs, InfAdd, InfCast, InfMul, InfSub, ProductOrd},
};

use super::{Float, Sequential, SumRelaxation};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type = "(T, T)")),
    generics(S(default = "Sequential<T>")),
    derived_types(T = "$get_atom_or_infer(S, get_first(bounds))")
)]
/// Make a Transformation that computes the sum of bounded floats with known ordering.
///
/// Only useful when `make_bounded_float_checked_sum` returns an error due to potential for overflow.
/// You may need to use `make_ordered_random` to impose an ordering on the data.
/// A smaller public `size_limit` reduces the rounding relaxation, which grows quadratically.
/// Construction fails if the rounding relaxation cannot be represented in the output type.
///
/// | S (summation algorithm) | input type     |
/// | ----------------------- | -------------- |
/// | `Sequential<S::Item>`   | `Vec<S::Item>` |
///
/// `S::Item` is the type of all of the following:
/// each bound, each element in the input data, the output data, and the output sensitivity.
///
/// Only sequential saturation is supported. Regrouping a saturating pairwise sum
/// can exceed its stability bound when an element is inserted or deleted.
/// For `f32` single-precision floats, set `S` to `Sequential<f32>`.
///
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
///
/// # Arguments
/// * `size_limit` - Upper bound on the number of records in input data. Used to bound sensitivity.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
///
/// # Generics
/// * `S` - Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
pub fn make_bounded_float_ordered_sum<S>(
    size_limit: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<S::Item>>,
        InsertDeleteDistance,
        AtomDomain<S::Item>,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: SaturatingSum,
    S::Item: 'static + Float,
{
    let (lower, upper) = bounds;
    let ideal_sensitivity = upper
        .inf_sub(&lower)?
        .total_max(lower.alerting_abs()?.total_max(upper)?)?;
    let relaxation = S::relaxation(size_limit, lower, upper)?;

    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?),
        InsertDeleteDistance,
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        Function::new(move |arg: &Vec<S::Item>| {
            S::saturating_sum(&arg[..size_limit.min(arg.len())])
        }),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            // Let BS be the exact sequential saturated fold and BS* its rounded implementation.
            // Clamp after each addition is nonexpansive, so a common suffix
            // cannot amplify an insertion/deletion difference.
            // d_out =  |BS*(v) - BS*(v')|
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in * ideal_sens + 2 * error
            //       =  d_in * ideal_sens + relaxation
            S::Item::inf_cast(*d_in)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    )
}

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type = "(T, T)")),
    generics(S(default = "Sequential<T>")),
    derived_types(T = "$get_atom_or_infer(S, get_first(bounds))")
)]
/// Make a Transformation that computes the sum of bounded floats with known ordering and dataset size.
///
/// Only useful when `make_bounded_float_checked_sum` returns an error due to potential for overflow.
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
/// You may need to use `make_ordered_random` to impose an ordering on the data.
///
/// | S (summation algorithm) | input type     |
/// | ----------------------- | -------------- |
/// | `Sequential<S::Item>`   | `Vec<S::Item>` |
///
/// `S::Item` is the type of all of the following:
/// each bound, each element in the input data, the output data, and the output sensitivity.
///
/// Only sequential saturation is supported. Regrouping a saturating pairwise sum
/// can exceed its stability bound when an element is inserted or deleted.
/// For `f32` single-precision floats, set `S` to `Sequential<f32>`.
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
/// * `S` - Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
pub fn make_sized_bounded_float_ordered_sum<S>(
    size: usize,
    bounds: (S::Item, S::Item),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<S::Item>>,
        InsertDeleteDistance,
        AtomDomain<S::Item>,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: SaturatingSum,
    S::Item: 'static + Float,
{
    let (lower, upper) = bounds;
    let ideal_sensitivity = upper.inf_sub(&lower)?;
    let relaxation = S::relaxation(size, lower, upper)?;

    Transformation::new(
        VectorDomain::new(AtomDomain::new_closed(bounds)?).with_size(size),
        InsertDeleteDistance,
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        Function::new(move |arg: &Vec<S::Item>| S::saturating_sum(arg)),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            // Let BS be the exact sequential saturated fold and BS* its rounded implementation.
            // Clamp after each addition is nonexpansive, so a common suffix
            // cannot amplify an insertion/deletion difference.
            // d_out =  |BS*(v) - BS*(v')|
            //       <= |BS*(v) - BS(v)| + |BS(v) - BS(v')| + |BS(v') - BS*(v')|
            //       <= d_in / 2 * (U - L) + 2 * error
            //       =  d_in / 2 * (U - L) + relaxation
            S::Item::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&relaxation)
        }),
    )
}

#[doc(hidden)]
pub trait SaturatingSum: SumRelaxation {
    fn saturating_sum(arg: &[Self::Item]) -> Self::Item;
}

impl<T: Float> SaturatingSum for Sequential<T> {
    fn saturating_sum(arg: &[T]) -> T {
        arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))
    }
}

#[cfg(test)]
mod test;
