#[cfg(feature = "ffi")]
mod ffi;

mod int;
pub use int::*;

mod float;
pub use float::*;
use opendp_derive::bootstrap;

use crate::core::{Metric, MetricSpace, Transformation};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::*;
use crate::metrics::{AbsoluteDistance, InsertDeleteDistance, SymmetricDistance};
use crate::traits::CheckAtom;
use crate::transformations::{make_ordered_random, make_unordered};
use int::signs_agree;

#[cfg(all(test, feature = "partials"))]
mod test;

#[bootstrap(features("contrib"), generics(MI(suppress), T(suppress)))]
/// Make a Transformation that computes the sum of bounded data.
/// Use `make_clamp` to bound data.
///
/// If dataset size is known, uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
///
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
///
/// # Arguments
/// * `input_domain` - Domain of the input data.
/// * `input_metric` - One of `SymmetricDistance` or `InsertDeleteDistance`.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
///
/// # Generics
/// * `MI` - Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`.
/// * `T` - Atomic Input Type and Output Type.
pub fn make_sum<MI, T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: MI,
) -> Fallible<Transformation<VectorDomain<AtomDomain<T>>, AtomDomain<T>, MI, AbsoluteDistance<T>>>
where
    MI: Metric,
    T: MakeSum<MI>,
    (VectorDomain<AtomDomain<T>>, MI): MetricSpace,
    (AtomDomain<T>, AbsoluteDistance<T>): MetricSpace,
{
    T::make_sum(input_domain, input_metric)
}

// make_sum

// implementations delegate to:
// make_(sized_)?bounded_int_(checked|monotonic|ordered|split)_sum
// make_(sized_)?bounded_float_(checked|ordered)_sum

#[doc(hidden)]
pub trait MakeSum<MI: Metric>: CheckAtom
where
    (VectorDomain<AtomDomain<Self>>, MI): MetricSpace,
    (AtomDomain<Self>, AbsoluteDistance<Self>): MetricSpace,
{
    /// # Proof Definition
    /// For any given input domain and input metric,
    /// returns `Ok(out)` where `out` is a valid transformation,
    /// or `Err(e)`.
    fn make_sum(
        input_domain: VectorDomain<AtomDomain<Self>>,
        input_metric: MI,
    ) -> Fallible<
        Transformation<
            VectorDomain<AtomDomain<Self>>,
            AtomDomain<Self>,
            MI,
            AbsoluteDistance<Self>,
        >,
    >;
}

macro_rules! impl_make_sum_int {
    ($($ty:ty)+) => {
        $(impl MakeSum<SymmetricDistance> for $ty {
            fn make_sum(
                input_domain: VectorDomain<AtomDomain<Self>>,
                _input_metric: SymmetricDistance,
            ) -> Fallible<Transformation<VectorDomain<AtomDomain<Self>>, AtomDomain<Self>, SymmetricDistance, AbsoluteDistance<Self>>> {
                let bounds = input_domain.element_domain.bounds
                    .ok_or_else(|| err!(MakeTransformation, "`input_domain` must be bounded. Use `make_clamp` to bound data."))?
                    .get_closed()?;

                if let Some(size) = input_domain.size {
                    if !can_int_sum_overflow(size, bounds) {
                        // 1. if the sum can't overflow, don't need to worry about saturation arithmetic
                        make_sized_bounded_int_checked_sum(size, bounds)

                    } else if signs_agree(bounds) {
                        // 2. a monotonic sum is less efficient due to saturation arithmetic
                        make_sized_bounded_int_monotonic_sum(size, bounds)

                    } else {
                        // 3. a split sum is the least efficient, because it needs saturation arithmetic and separate pos/neg sums
                        make_sized_bounded_int_split_sum(size, bounds)
                    }
                } else {
                    // data size unknown, so checked sum is not applicable
                    if signs_agree(bounds) {
                        // 1. if bounds share sign, then a simple saturating addition is associative
                        make_bounded_int_monotonic_sum(bounds)

                    } else {
                        // 2. split sum is the cheapest remaining fallback when order is unknown
                        make_bounded_int_split_sum(bounds)
                    }
                }
            }
        })+
        $(impl MakeSum<InsertDeleteDistance> for $ty {
            fn make_sum(
                input_domain: VectorDomain<AtomDomain<Self>>,
                input_metric: InsertDeleteDistance,
            ) -> Fallible<Transformation<VectorDomain<AtomDomain<Self>>, AtomDomain<Self>, InsertDeleteDistance, AbsoluteDistance<Self>>> {
                let bounds = input_domain.element_domain.bounds
                    .ok_or_else(|| err!(MakeTransformation, "`input_domain` must be bounded. Use `make_clamp` to bound data."))?
                    .get_closed()?;

                if let Some(size) = input_domain.size {
                    if !can_int_sum_overflow(size, bounds) {
                        // 1. if the sum can't overflow,
                        //    then do a no-op unordering and use a more computationally efficient sum without saturation arithmetic
                        let domain = VectorDomain::new(AtomDomain::new_closed(bounds)?).with_size(size);
                        make_unordered(domain, input_metric)? >> make_sized_bounded_int_checked_sum(size, bounds)?

                    } else {
                        // when input metric is ordered, ordered sum doesn't need a shuffle, making it comparatively cheaper
                        // - ordered sum is more efficient than a split sum because splitting unnecessary
                        // - ordered sum is tied with monotonic sum, but not conditional on monotonicity
                        make_sized_bounded_int_ordered_sum(size, bounds)
                    }
                } else {
                    // data size unknown, so checked sum is not applicable

                    // when input metric is ordered, ordered sum doesn't need a shuffle, making it comparatively cheaper
                    // - ordered sum is more efficient than a split sum because splitting unnecessary
                    // - ordered sum is tied with monotonic sum, but not conditional on monotonicity
                    make_bounded_int_ordered_sum(bounds)
                }
            }
        })+
    };
}
impl_make_sum_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }

const DEFAULT_SIZE_LIMIT: usize = 1_048_576; // 2^20
macro_rules! impl_make_sum_float {
    ($($ty:ty)+) => {
        $(impl MakeSum<SymmetricDistance> for $ty {
            fn make_sum(
                input_domain: VectorDomain<AtomDomain<Self>>,
                input_metric: SymmetricDistance,
            ) -> Fallible<Transformation<VectorDomain<AtomDomain<Self>>, AtomDomain<Self>, SymmetricDistance, AbsoluteDistance<Self>>> {
                let bounds = input_domain.element_domain.bounds.as_ref()
                    .ok_or_else(|| err!(MakeTransformation, "`input_domain` must be bounded. Use `make_clamp` to bound data."))?
                    .get_closed()?;

                if let Some(size) = input_domain.size {
                    if !Pairwise::<Self>::can_float_sum_overflow(size, bounds)? {
                        // 1. try the checked sum first, as floats are unlikely to overflow
                        make_sized_bounded_float_checked_sum::<Pairwise<_>>(size, bounds)

                    } else {
                        // 2. fall back to ordered summation
                        make_ordered_random(input_domain, input_metric)? >> make_sized_bounded_float_ordered_sum::<Pairwise<_>>(size, bounds)?
                    }
                } else {
                    if !Pairwise::<Self>::can_float_sum_overflow(DEFAULT_SIZE_LIMIT, bounds)? {
                        // 1. if the sum can't overflow, then use a more computationally efficient sum without saturation arithmetic
                        make_bounded_float_checked_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)

                    } else {
                        // 2. sum can overflow, so shuffle and use an ordered sum
                        make_ordered_random(input_domain, input_metric)? >> make_bounded_float_ordered_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)?
                    }
                }
            }
        })+

        $(impl MakeSum<InsertDeleteDistance> for $ty {
            fn make_sum(
                input_domain: VectorDomain<AtomDomain<Self>>,
                input_metric: InsertDeleteDistance,
            ) -> Fallible<Transformation<VectorDomain<AtomDomain<Self>>, AtomDomain<Self>, InsertDeleteDistance, AbsoluteDistance<Self>>> {
                let bounds = input_domain.element_domain.bounds.as_ref()
                    .ok_or_else(|| err!(MakeTransformation, "`input_domain` must be bounded. Use `make_clamp` to bound data."))?
                    .get_closed()?;

                if let Some(size) = input_domain.size {
                    if !Pairwise::<Self>::can_float_sum_overflow(size, bounds)? {
                        // 1. if the sum can't overflow,
                        //    then do a no-op unordering and use a more computationally efficient sum without saturation arithmetic
                        make_unordered(input_domain, input_metric)? >> make_sized_bounded_float_checked_sum::<Pairwise<_>>(size, bounds)?

                    } else {
                        // 2. fall back to ordered summation
                        make_sized_bounded_float_ordered_sum::<Pairwise<_>>(size, bounds)
                    }
                } else {
                    if !Pairwise::<Self>::can_float_sum_overflow(DEFAULT_SIZE_LIMIT, bounds)? {
                        // 1. if the sum can't overflow,
                        //    then do a no-op unordering and use a more computationally efficient sum without saturation arithmetic
                        make_unordered(input_domain, input_metric)? >> make_bounded_float_checked_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)?

                    } else {
                        // 2. sum can overflow, but data is already sorted, so use a saturating sum
                        make_bounded_float_ordered_sum::<Pairwise<_>>(DEFAULT_SIZE_LIMIT, bounds)
                    }
                }
            }
        })+
    };
}
impl_make_sum_float! { f32 f64 }
