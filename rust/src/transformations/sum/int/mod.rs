use crate::{
    core::{Metric, StabilityMap},
    error::Fallible,
    metrics::{AbsoluteDistance, IntDistance},
    traits::{Integer, SaturatingCast},
};
use dashu::integer::IBig;
use num::Signed;

// Largest length for which every possible intermediate sum is representable.
// Both quotients are nonnegative; division and MIN handling are entirely in IBig.
pub(super) fn size_limit<T: Integer>((lower, upper): (T, T)) -> usize
where
    IBig: From<T>,
{
    let mut limit = <IBig as From<usize>>::from(usize::MAX);
    let lower = IBig::from(lower);
    let upper = IBig::from(upper);
    if upper > IBig::ZERO {
        limit = limit.min(IBig::from(T::MAX_FINITE) / upper);
    }
    if lower < IBig::ZERO {
        limit = limit.min(IBig::from(T::MIN_FINITE) / lower);
    }
    usize::saturating_cast(limit)
}

// Used by the Polars integer accumulator as well as standalone sums.
// Includes every prefix length k in [0, size], and never takes native abs(MIN).
pub(crate) fn can_int_sum_overflow<T: Integer>(size: usize, bounds: (T, T)) -> bool
where
    IBig: From<T>,
{
    size > size_limit(bounds)
}

pub(super) fn stability_map<MI, T>(
    n: usize,
    (lower, upper): (T, T),
    fixed: bool,
) -> Fallible<StabilityMap<MI, AbsoluteDistance<T>>>
where
    MI: Metric<Distance = IntDistance>,
    T: Integer + TryFrom<IBig>,
    IBig: From<T>,
{
    if can_int_sum_overflow(n, (lower, upper)) {
        return fallible!(
            MakeTransformation,
            "potential intermediate integer sum overflow; tighten bounds or length"
        );
    }
    let lower = IBig::from(lower);
    let upper = IBig::from(upper);
    let sensitivity = if fixed {
        &upper - &lower
    } else {
        lower.abs().max(upper.abs()).max(upper - lower)
    };
    Ok(StabilityMap::new_fallible(move |d: &IntDistance| {
        let d = if fixed { *d / 2 } else { *d };
        T::try_from(<IBig as From<IntDistance>>::from(d) * &sensitivity).map_err(|_| {
            err!(
                Overflow,
                "integer sum sensitivity is not representable in the output distance type"
            )
        })
    }))
}
