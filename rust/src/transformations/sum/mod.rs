#[cfg(feature = "ffi")]
mod ffi;

mod int;
#[cfg(feature = "polars")]
pub(crate) use int::size_limit as int_sum_size_limit;
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
/// Make a Transformation that computes the sum of bounded data.
/// Use `make_clamp` to bound data. Bounds must exclude NaN and be finite.
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
///
/// # Generics
/// * `MI` - Input Metric. One of `SymmetricDistance` or `InsertDeleteDistance`.
/// * `T` - Atomic Input Type and Output Type.
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
