use crate::{
    core::{Metric, StabilityMap},
    error::Fallible,
    metrics::{AbsoluteDistance, IntDistance},
    traits::{
        CastInternalRational, Float, InfCast,
        sum::{round_up, sum_magnitude, sum_relaxation},
    },
};
use dashu::{integer::IBig, rational::RBig};
use std::marker::PhantomData;

/// Left-to-right floating-point summation.
pub struct Sequential<T>(PhantomData<T>);
/// Recursive-halving pairwise floating-point summation (the default).
pub struct Pairwise<T>(PhantomData<T>);

/// Execution and reduction depth, reusable without vector-domain semantics.
#[doc(hidden)]
pub trait UncheckedSum {
    type Item: Float;
    /// Maximum number of rounded additions on a leaf-to-root path.
    fn depth(size: usize) -> usize;
    fn unchecked_sum(arg: &[Self::Item]) -> Self::Item;
}
impl<T: Float> UncheckedSum for Sequential<T> {
    type Item = T;
    fn depth(size: usize) -> usize {
        size.saturating_sub(1)
    }
    fn unchecked_sum(arg: &[T]) -> T {
        arg.iter().sum()
    }
}
impl<T: Float> UncheckedSum for Pairwise<T> {
    type Item = T;
    fn depth(size: usize) -> usize {
        if size <= 1 {
            0
        } else {
            (usize::BITS - (size - 1).leading_zeros()) as usize
        }
    }
    fn unchecked_sum(arg: &[T]) -> T {
        match arg.len() {
            0 => T::zero(),
            1 => arg[0],
            n => {
                let m = n / 2;
                Self::unchecked_sum(&arg[..m]) + Self::unchecked_sum(&arg[m..])
            }
        }
    }
}

pub(super) fn stability_map<MI, S>(
    max_size: usize,
    (lower, upper): (S::Item, S::Item),
    fixed: bool,
) -> Fallible<StabilityMap<MI, AbsoluteDistance<S::Item>>>
where
    MI: Metric<Distance = IntDistance>,
    S: UncheckedSum,
    S::Item: InfCast<RBig>,
{
    let magnitude = sum_magnitude((lower, upper))?;
    let relaxation = sum_relaxation(max_size, S::depth(max_size), magnitude)?;
    let range = upper.into_rational()? - lower.into_rational()?;
    let sensitivity = if fixed {
        range
    } else {
        magnitude.into_rational()?.max(range)
    };
    Ok(StabilityMap::new_fallible(move |d: &IntDistance| {
        let d = if fixed { *d / 2 } else { *d };
        // Keep both endpoint errors even at d=0: symmetric distance identifies
        // permutations, but floating-point algorithms may distinguish them.
        round_up::<S::Item>(RBig::from(IBig::from(d)) * &sensitivity + &relaxation)
    }))
}
