#[cfg(feature = "ffi")]
mod ffi;

mod int;
#[cfg(feature = "polars")]
pub(crate) use int::can_int_sum_overflow;
mod float;
pub use float::{Pairwise, Sequential, UncheckedSum};
mod selection;
use selection::select_contributions;

use crate::{
    core::{Function, Metric, MetricSpace, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, InsertDeleteDistance, IntDistance, SymmetricDistance},
    traits::Number,
};
use opendp_derive::bootstrap;

#[cfg(test)]
mod test;

const DEFAULT_SIZE_LIMIT: usize = 1_048_576;

#[bootstrap(features("contrib"), generics(MI(suppress), T(suppress)))]
#[deprecated(
    note = "Use Polars or NumPy aggregation APIs instead.",
    since = "0.16.0"
)]
/// Make a Transformation that sums bounded values with constructor-validated native arithmetic.
///
/// Closed element bounds excluding NaN are required, and floating bounds must be finite.
/// Integers use native accumulation; floats use pairwise accumulation.
/// Exact-size domains are summed in full when numerically safe. For unsized
/// integers, the maximum safe retained length is derived from the element bounds
/// and the integer representation; for unsized floats it is 2^20. Ordered data
/// uses its prefix, and oversized unordered data is uniformly sampled without
/// replacement.
///
/// Unsized integer inputs may now truncate, unlike the original implementation.
/// The retained length does not restrict domain membership.
/// Construction establishes safe accumulation of every retained record.
/// Floating stability includes rounding relaxation even at symmetric distance zero.
///
/// # Arguments
/// * `input_domain` - Vector domain with bounded elements.
/// * `input_metric` - One of `SymmetricDistance` or `InsertDeleteDistance`.
///
/// # Generics
/// * `MI` - Input dataset metric.
/// * `T` - Atomic input and output type, inferred from the input domain.
pub fn make_sum<MI, T>(
    input_domain: VectorDomain<AtomDomain<T>>,
    input_metric: MI,
) -> Fallible<Transformation<VectorDomain<AtomDomain<T>>, MI, AtomDomain<T>, AbsoluteDistance<T>>>
where
    MI: Metric,
    T: MakeSum<MI>,
    (VectorDomain<AtomDomain<T>>, MI): MetricSpace,
{
    input_domain.element_domain.assert_non_nan()?;
    let bounds = input_domain.element_domain.get_closed_bounds()?;
    let n = input_domain.size.unwrap_or_else(|| T::size_limit(bounds));
    let fixed = input_domain.size.is_some();
    let stability_map = T::stability_map(n, bounds, fixed)?;
    Transformation::<_, _, AtomDomain<T>, AbsoluteDistance<T>>::new(
        input_domain,
        input_metric,
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        Function::new_fallible(move |arg: &Vec<T>| {
            let selected = select_contributions(arg, n, T::ORDERED)?;
            Ok(<T as MakeSum<MI>>::sum(&selected))
        }),
        stability_map,
    )
}

#[doc(hidden)]
pub trait MakeSum<MI: Metric>: Number {
    const ORDERED: bool;
    fn size_limit(bounds: (Self, Self)) -> usize;
    fn sum(arg: &[Self]) -> Self;
    fn stability_map(
        max_size: usize,
        bounds: (Self, Self),
        fixed: bool,
    ) -> Fallible<StabilityMap<MI, AbsoluteDistance<Self>>>;
}

// Only add/remove dataset metrics have the sensitivity formulas proved here.
pub(crate) trait SumMetric: Metric<Distance = IntDistance> {
    const ORDERED: bool;
}
impl SumMetric for SymmetricDistance {
    const ORDERED: bool = false;
}
impl SumMetric for InsertDeleteDistance {
    const ORDERED: bool = true;
}

macro_rules! impl_make_sum_int {
    ($($ty:ty)+) => {$ (
        impl<MI: SumMetric> MakeSum<MI> for $ty {
            const ORDERED: bool = MI::ORDERED;
            fn size_limit(bounds: (Self, Self)) -> usize { int::size_limit(bounds) }
            fn sum(arg: &[Self]) -> Self {
                arg.iter().copied().sum()
            }
            fn stability_map(n: usize, bounds: (Self, Self), fixed: bool)
                -> Fallible<StabilityMap<MI, AbsoluteDistance<Self>>>
            { int::stability_map(n, bounds, fixed) }
        }
    )+};
}
impl_make_sum_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize }

macro_rules! impl_make_sum_float {
    ($($ty:ty)+) => {$ (
        impl<MI: SumMetric> MakeSum<MI> for $ty {
            const ORDERED: bool = MI::ORDERED;
            fn size_limit(_bounds: (Self, Self)) -> usize { DEFAULT_SIZE_LIMIT }
            fn sum(arg: &[Self]) -> Self {
                Pairwise::<Self>::unchecked_sum(arg)
            }
            fn stability_map(n: usize, bounds: (Self, Self), fixed: bool)
                -> Fallible<StabilityMap<MI, AbsoluteDistance<Self>>>
            { float::stability_map::<MI, Pairwise<Self>>(n, bounds, fixed) }
        }
    )+};
}
impl_make_sum_float! { f32 f64 }
