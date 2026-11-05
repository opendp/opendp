use std::{
    cell::Cell,
    cmp::Ordering,
    iter::{once, successors},
    mem::swap,
    ops::{Add, Div, Sub},
};

use num::{CheckedAdd, CheckedSub, One, Zero};

use crate::{
    error::{Error, ErrorVariant, Fallible},
    traits::{ExactIntCast, FiniteBounds},
};

#[cfg(test)]
mod test;

/// Types that support OpenDP's binary-search utilities.
pub trait BinarySearchable:
    Bands
    + FiniteBounds
    + Zero
    + One
    + Clone
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
{
    fn midpoint(lower: &Self, upper: &Self) -> Self;
}

macro_rules! impl_binary_searchable_float {
    ($($ty:ty),+ $(,)?) => {
        $(impl BinarySearchable for $ty {
            fn midpoint(lower: &Self, upper: &Self) -> Self {
                let sign = (-0.0 as $ty).to_bits();
                // Reverse negative magnitudes and offset positives so ranks follow
                // numerical order, with both signed zeros sharing the same rank.
                let rank = |value: &$ty| {
                    let bits = value.to_bits();
                    if bits & sign != 0 {
                        sign - (bits & !sign)
                    } else {
                        sign + bits
                    }
                };
                let lower = rank(lower);
                let upper = rank(upper);
                let mid = lower + (upper - lower) / 2;
                <$ty>::from_bits(if mid < sign {
                    sign | (sign - mid)
                } else {
                    mid - sign
                })
            }
        })+
    };
}
impl_binary_searchable_float!(f32, f64);

macro_rules! impl_binary_searchable_int {
    ($($ty:ty),+ $(,)?) => {
        $(impl BinarySearchable for $ty {
            fn midpoint(lower: &Self, upper: &Self) -> Self {
                // The midpoint calculation differs
                // depending on whether the int is signed, to avoid overflow
                if lower < &<$ty>::zero() && upper >= &<$ty>::zero() {
                    (lower + upper).halve()
                } else {
                    lower + (upper - lower).halve()
                }
            }
        })+
    };
}
impl_binary_searchable_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

pub trait Halve {
    fn halve(&self) -> Self;
}

impl<T> Halve for T
where
    T: One + Add<Output = T>,
    for<'a> &'a T: Div<T, Output = T>,
{
    fn halve(&self) -> Self {
        self / (T::one() + T::one())
    }
}

mod private {
    use super::{Above, Below};
    pub trait Sealed<T> {}

    impl<T> Sealed<T> for () {}
    impl<T> Sealed<T> for (T, T) {}
    impl<T> Sealed<T> for Above<T> {}
    impl<T> Sealed<T> for Below<T> {}
    impl<T> Sealed<T> for (Option<T>, Option<T>) {}
    impl<T> Sealed<T> for Option<(T, T)> {}
}

pub trait BoundSpec<T>: private::Sealed<T> {
    fn resolve(self) -> (Option<T>, Option<T>);
}

pub struct Above<T>(pub T);
pub struct Below<T>(pub T);

impl<T> BoundSpec<T> for () {
    fn resolve(self) -> (Option<T>, Option<T>) {
        (None, None)
    }
}

impl<T> BoundSpec<T> for (T, T) {
    fn resolve(self) -> (Option<T>, Option<T>) {
        (Some(self.0), Some(self.1))
    }
}

impl<T> BoundSpec<T> for Above<T> {
    fn resolve(self) -> (Option<T>, Option<T>) {
        (Some(self.0), None)
    }
}

impl<T> BoundSpec<T> for Below<T> {
    fn resolve(self) -> (Option<T>, Option<T>) {
        (None, Some(self.0))
    }
}
impl<T> BoundSpec<T> for (Option<T>, Option<T>) {
    fn resolve(self) -> (Option<T>, Option<T>) {
        self
    }
}

impl<T> BoundSpec<T> for Option<(T, T)> {
    fn resolve(self) -> (Option<T>, Option<T>) {
        match self {
            Some((lower, upper)) => (Some(lower), Some(upper)),
            None => (None, None),
        }
    }
}

/// Find the closest passing value to the decision boundary of `predicate`.
///
/// Missing bounds are inferred:
/// - if neither bound is passed, an exponential search infers both bounds
/// - if only `lower` is passed, a band search infers `upper`
/// - if only `upper` is passed, a band search infers `lower`
pub fn binary_search<T>(predicate: impl Fn(&T) -> bool, bounds: impl BoundSpec<T>) -> Fallible<T>
where
    T: BinarySearchable,
{
    signed_binary_search(predicate, bounds).map(|(value, _sign)| value)
}

/// Like [`binary_search`], but also returns the direction away from the decision boundary.
///
/// A returned sign of `1` means the passing side is above the boundary, and `-1` means it is below.
pub fn signed_binary_search<T>(
    predicate: impl Fn(&T) -> bool,
    bounds: impl BoundSpec<T>,
) -> Fallible<(T, i8)>
where
    T: BinarySearchable,
{
    let predicate = move |value: &T| Ok(predicate(value));
    signed_fallible_binary_search(predicate, bounds)
}

/// Fallible version of [`binary_search`].
pub fn fallible_binary_search<T>(
    predicate: impl Fn(&T) -> Fallible<bool>,
    bounds: impl BoundSpec<T>,
) -> Fallible<T>
where
    T: BinarySearchable,
{
    signed_fallible_binary_search(predicate, bounds).map(|(value, _sign)| value)
}

/// Fallible version of [`signed_binary_search`].
pub fn signed_fallible_binary_search<T>(
    predicate: impl Fn(&T) -> Fallible<bool>,
    bounds: impl BoundSpec<T>,
) -> Fallible<(T, i8)>
where
    T: BinarySearchable,
{
    let bounds = resolve_bounds(&predicate, bounds)?;
    signed_fallible_binary_search_with_bounds(&predicate, bounds)
}

fn resolve_bounds<T>(
    predicate: &impl Fn(&T) -> Fallible<bool>,
    bounds: impl BoundSpec<T>,
) -> Fallible<((T, bool), (T, bool))>
where
    T: BinarySearchable,
{
    let (lower, upper) = bounds.resolve();
    for value in lower.iter().chain(upper.iter()) {
        if !(value >= &T::MIN_FINITE && value <= &T::MAX_FINITE) {
            return fallible!(Search, "search bounds must be finite and not NaN");
        }
    }
    let (mut lower, mut upper) = match (lower, upper) {
        (Some(lower), Some(upper)) => (lower, upper),
        (Some(lower), None) => {
            return infer_bounds(predicate, lower, &[1])?.ok_or_else(|| {
                err!(
                    Search,
                    "the decision boundary is below the lower bound or the predicate does not change above it"
                )
            });
        }
        (None, Some(upper)) => {
            return infer_bounds(predicate, upper, &[-1])?.ok_or_else(|| {
                err!(
                    Search,
                    "the decision boundary is above the upper bound or the predicate does not change below it"
                )
            });
        }
        (None, None) => {
            return infer_bounds(predicate, T::zero(), &[1, -1])?
                .ok_or_else(|| err!(Search, "unable to infer bounds"));
        }
    };
    if lower > upper {
        swap(&mut lower, &mut upper);
    }
    // Evaluate sequentially: unexpected errors must escape before any further
    // callback. Pass the observations to bisection rather than evaluating twice.
    let maximize = match predicate(&lower) {
        Ok(value) => value,
        Err(error) if is_range_error(&error) => {
            let at_upper = predicate(&upper)?;
            return recover_range_bracket(predicate, upper, at_upper, lower, error);
        }
        Err(error) => return Err(error),
    };
    let minimize = match predicate(&upper) {
        Ok(value) => value,
        Err(error) if is_range_error(&error) => {
            return recover_range_bracket(predicate, lower, maximize, upper, error);
        }
        Err(error) => return Err(error),
    };
    Ok(((lower, maximize), (upper, minimize)))
}

fn signed_fallible_binary_search_with_bounds<T>(
    predicate: &dyn Fn(&T) -> Fallible<bool>,
    bounds: ((T, bool), (T, bool)),
) -> Fallible<(T, i8)>
where
    T: BinarySearchable,
{
    let (mut lower, mut upper) = bounds;
    if lower.0 > upper.0 {
        swap(&mut lower, &mut upper);
    }
    let ((mut lower, maximize), (mut upper, minimize)) = (lower, upper);

    if maximize == minimize {
        return fallible!(
            Search,
            "the decision boundary of the predicate is outside the bounds"
        );
    }

    let mut mid = lower.clone();

    loop {
        let new_mid = T::midpoint(&lower, &upper);

        // Avoid an infinite loop from float roundoff or integer truncation.
        if new_mid == mid || new_mid == lower || new_mid == upper {
            break;
        }

        mid = new_mid;
        if predicate(&mid)? == minimize {
            upper = mid.clone();
        } else {
            lower = mid.clone();
        }
    }

    Ok((
        if minimize { upper } else { lower },
        if minimize { 1 } else { -1 },
    ))
}

pub trait Bands: Sized {
    fn bands(center: Self, sign: i8) -> Vec<Self>;
}

macro_rules! impl_bands_float {
    ($($ty:ty),+ $(,)?) => {
        $(impl Bands for $ty {
            fn bands(center: Self, sign: i8) -> Vec<Self> {
                let sign: Self = if sign > 0 { 1.0 } else { -1.0 };
                let two: Self = 2.0;
                let mut bands = vec![center];
                let offsets = once(0.5).chain((0i32..)
                    .map(|k| k * k)
                    .take_while(|&exponent| exponent < Self::MAX_EXP)
                    .map(|exponent| two.powi(exponent)));
                for offset in offsets {
                    let candidate = center + sign * offset;
                    if candidate.is_finite() && sign * candidate > sign * *bands.last().unwrap() {
                        bands.push(candidate);
                    }
                }
                let extreme = sign * Self::MAX;
                if bands.last() != Some(&extreme) {
                    bands.push(extreme);
                }
                bands
            }
        })+
    };
}
impl_bands_float!(f32, f64);

fn band_offsets() -> impl Iterator<Item = u128> {
    once(1u128).chain(successors(Some(16u128), |x| x.checked_mul(16)))
}

fn bands_int<T>(center: T, sign: i8) -> Vec<T>
where
    T: Copy + PartialEq + FiniteBounds + CheckedAdd + CheckedSub + ExactIntCast<u128>,
{
    let upward = sign > 0;
    let mut bands = vec![center];

    for offset in band_offsets() {
        let Ok(offset) = T::exact_int_cast(offset) else {
            break;
        };

        let candidate = if upward {
            center.checked_add(&offset)
        } else {
            center.checked_sub(&offset)
        };

        let Some(candidate) = candidate else {
            break;
        };

        bands.push(candidate);
    }

    let extreme = if upward { T::MAX_FINITE } else { T::MIN_FINITE };
    if bands.last() != Some(&extreme) {
        bands.push(extreme);
    }

    bands
}

macro_rules! impl_bands_int {
    ($($ty:ty),+ $(,)?) => {
        $(impl Bands for $ty {
            fn bands(center: Self, sign: i8) -> Vec<Self> {
                bands_int(center, sign)
            }
        })+
    };
}

impl_bands_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

/// Determine bounds for a binary search via an exponential search.
pub fn exponential_bounds_search<T>(predicate: &impl Fn(&T) -> bool) -> Option<(T, T)>
where
    T: BinarySearchable,
{
    let center = T::zero();
    let at_center = predicate(&center);

    fallible_exponential_bounds_search_from(
        &|value| Ok(predicate(value)),
        center,
        at_center,
        &[1, -1],
    )
    .ok()
    .flatten()
    .map(|((lower, _), (upper, _))| (lower, upper))
}

/// Determine bounds for a binary search via an exponential search.
///
/// Overflow makes a probe unavailable, not passing or failing. The predicate must
/// be deterministic and monotone on its evaluable interval. Any initial error
/// triggers best-effort finite anchor discovery, which can miss narrow intervals.
/// Once anchored, unexpected errors propagate outside the initial invalid segment.
pub fn fallible_exponential_bounds_search<T>(
    predicate: &impl Fn(&T) -> Fallible<bool>,
) -> Fallible<Option<(T, T)>>
where
    T: BinarySearchable,
{
    infer_bounds(predicate, T::zero(), &[1, -1])
        .map(|bounds| bounds.map(|((lower, _), (upper, _))| (lower, upper)))
}

fn infer_bounds<T: BinarySearchable>(
    predicate: &impl Fn(&T) -> Fallible<bool>,
    center: T,
    directions: &[i8],
) -> Fallible<Option<((T, bool), (T, bool))>> {
    let initial_error = match predicate(&center) {
        Ok(value) => {
            return fallible_exponential_bounds_search_from(predicate, center, value, directions);
        }
        Err(error) => error,
    };
    // Best effort only: these finite probes may miss a narrow evaluable interval.
    let bands: Vec<_> = directions
        .iter()
        .map(|&sign| T::bands(center.clone(), sign))
        .collect();
    let mut anchored = vec![false; bands.len()];
    for index in 1..bands.iter().map(Vec::len).max().unwrap_or(0) {
        for (direction, bands) in bands.iter().enumerate() {
            if anchored[direction] {
                continue;
            }
            let Some(point) = bands.get(index) else {
                continue;
            };
            if let Ok(value) = predicate(point) {
                if let Some(bounds) =
                    recover_initial_segment(predicate, center.clone(), point.clone(), value)?
                {
                    return Ok(Some(bounds));
                }
                // This direction was evaluable but had no Boolean bracket.
                anchored[direction] = true;
            }
        }
    }
    if anchored.contains(&true) {
        Ok(None)
    } else {
        Err(initial_error)
    }
}

// Only this initial segment can contain arbitrary domain/construction failures.
// Successful observations, never errors, establish the predicate bracket.
fn recover_initial_segment<T: BinarySearchable>(
    predicate: &impl Fn(&T) -> Fallible<bool>,
    mut unavailable: T,
    mut anchor: T,
    at_anchor: bool,
) -> Fallible<Option<((T, bool), (T, bool))>> {
    let sign = if anchor > unavailable { 1 } else { -1 };
    loop {
        let mid = if sign > 0 {
            T::midpoint(&unavailable, &anchor)
        } else {
            T::midpoint(&anchor, &unavailable)
        };
        if mid == anchor || mid == unavailable {
            break;
        }
        match predicate(&mid) {
            Ok(value) if value != at_anchor => {
                return Ok(Some(((anchor, at_anchor), (mid, value))));
            }
            Ok(_) => anchor = mid,
            Err(_) => unavailable = mid,
        }
    }
    // Confirm the evaluable endpoint before leaving initial recovery.
    predicate(&anchor)?;
    fallible_exponential_bounds_search_from(predicate, anchor, at_anchor, &[sign])
}

fn fallible_exponential_bounds_search_from<T: BinarySearchable>(
    predicate: &dyn Fn(&T) -> Fallible<bool>,
    center: T,
    at_center: bool,
    directions: &[i8],
) -> Fallible<Option<((T, bool), (T, bool))>> {
    let mut range_error = None;
    // Try one direction at a time. For a successful zero anchor this finds
    // ordinary positive calibrations before probing invalid negative parameters.
    for &sign in directions {
        match fallible_band_search(predicate, &T::bands(center.clone(), sign), at_center) {
            Ok(Some(bounds)) => return Ok(Some(bounds)),
            Ok(None) => (),
            Err(error) if is_range_error(&error) => range_error = Some(error),
            Err(error) => return Err(error),
        }
    }
    match range_error {
        Some(error) => Err(error),
        None => Ok(None),
    }
}

fn fallible_band_search<T: BinarySearchable>(
    predicate: &dyn Fn(&T) -> Fallible<bool>,
    bands: &[T],
    at_center: bool,
) -> Fallible<Option<((T, bool), (T, bool))>> {
    for window in bands.windows(2) {
        match predicate(&window[1]) {
            Ok(value) if value != at_center => {
                let mut lower = (window[0].clone(), at_center);
                let mut upper = (window[1].clone(), value);
                if lower.0 > upper.0 {
                    swap(&mut lower, &mut upper);
                }
                return Ok(Some((lower, upper)));
            }
            Ok(_) => (),
            Err(error) if is_range_error(&error) => {
                return recover_range_bracket(
                    predicate,
                    window[0].clone(),
                    at_center,
                    window[1].clone(),
                    error,
                )
                .map(Some);
            }
            Err(error) => return Err(error),
        }
    }

    Ok(None)
}

fn is_range_error(error: &Error) -> bool {
    error.variant == ErrorVariant::Overflow
}

/// Recover only along the segment from a successful observation to an unavailable
/// probe. Use the Boolean engine to find the evaluation edge; overflow itself
/// supplies no information about the original Boolean predicate.
fn recover_range_bracket<T: BinarySearchable>(
    predicate: &dyn Fn(&T) -> Fallible<bool>,
    center: T,
    at_center: bool,
    unavailable: T,
    error: Error,
) -> Fallible<((T, bool), (T, bool))> {
    let at_edge = Cell::new(at_center);
    let (edge, _) = signed_fallible_binary_search_with_bounds(
        &|value| match predicate(value) {
            Ok(observation) => {
                at_edge.set(observation);
                Ok(true)
            }
            Err(error) if is_range_error(&error) => Ok(false),
            Err(error) => Err(error),
        },
        ((center.clone(), true), (unavailable, false)),
    )?;
    // Bisection's passing endpoint is its last successful evaluation.
    let at_edge = at_edge.get();
    if at_edge == at_center {
        return Err(error);
    }
    Ok(if center < edge {
        ((center, at_center), (edge, at_edge))
    } else {
        ((edge, at_edge), (center, at_center))
    })
}

/// Search direction for scalar objectives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchMode {
    Minimize,
    Maximize,
}

impl SearchMode {
    #[inline]
    pub(crate) fn is_better(self, candidate: f64, incumbent: f64) -> bool {
        match self {
            SearchMode::Minimize => candidate < incumbent,
            SearchMode::Maximize => candidate > incumbent,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OptimizationResult {
    pub(crate) arg: f64,
    pub(crate) value: f64,
}

/// Heuristically optimize a unimodal f64 objective over `[lo, hi]` to
/// floating-point precision, returning the best of the final candidates.
///
/// This is not a certified optimum or objective bound: rounding, approximate
/// evaluations, or flat comparisons may invalidate golden-section exclusions.
/// Callers needing certification must bound the original domain independently.
/// A NaN objective value is rejected rather than assigned a direction-specific
/// meaning. Callers that use NaN as a heuristic sentinel should handle it
/// before invoking this primitive.
#[inline]
pub(crate) fn fallible_optimize_unimodal<F>(
    mode: SearchMode,
    lo: f64,
    hi: f64,
    f: F,
) -> Fallible<OptimizationResult>
where
    F: Fn(f64) -> Fallible<f64>,
{
    let (lo, hi) = fallible_golden_section_bracket_by(mode, lo, hi, &f, |left, right| {
        left.partial_cmp(right)
            .ok_or_else(|| err!(NumericError, "optimization objective returned NaN"))
    })?;

    let mut best = evaluate_candidate(lo, &f)?;
    for arg in [hi, interpolate(lo, hi, 0.5)] {
        let candidate = evaluate_candidate(arg, &f)?;
        if mode.is_better(candidate.value, best.value) {
            best = candidate;
        }
    }
    Ok(best)
}

fn evaluate_candidate<F>(arg: f64, f: &F) -> Fallible<OptimizationResult>
where
    F: Fn(f64) -> Fallible<f64>,
{
    let value = f(arg)?;
    if value.is_nan() {
        return fallible!(NumericError, "optimization objective returned NaN");
    }
    Ok(OptimizationResult { arg, value })
}

/// Perform golden-section search while comparing generic, ordered objective
/// values, returning the remaining heuristic coordinate interval.
///
/// Excluding regions requires a strictly unimodal objective and comparisons
/// that preserve its ordering; neither assumption is checked here. In
/// particular, exact comparisons of approximate objective values do not certify
/// an enclosure of a true optimizer. Do not use this interval as a certificate
/// without separately establishing those assumptions.
///
/// The callback is fallible so callers can preserve errors from ordered
/// numeric backends instead of converting objective values to f64.
#[inline]
pub(crate) fn fallible_golden_section_bracket_by<V, F, Compare>(
    mode: SearchMode,
    mut lo: f64,
    mut hi: f64,
    f: F,
    compare: Compare,
) -> Fallible<(f64, f64)>
where
    F: Fn(f64) -> Fallible<V>,
    Compare: Fn(&V, &V) -> Fallible<Ordering>,
{
    const INV_PHI: f64 = 0.6180339887498949;
    const INV_PHI2: f64 = 0.3819660112501051;

    if !lo.is_finite() || !hi.is_finite() || lo > hi {
        return fallible!(Search, "optimization bounds must be finite and ordered");
    }
    if lo == hi {
        f(lo)?;
        return Ok((lo, hi));
    }

    let mut c = interpolate(lo, hi, INV_PHI2);
    let mut d = interpolate(lo, hi, INV_PHI);
    let mut fc = f(c)?;
    let mut fd = f(d)?;

    loop {
        // Cached coordinates can drift out of order through rounding. Refresh
        // them before deciding the bracket has no usable interior coordinates.
        if !(lo < c && c < d && d < hi) {
            c = interpolate(lo, hi, INV_PHI2);
            d = interpolate(lo, hi, INV_PHI);
            if !(lo < c && c < d && d < hi) {
                break;
            }
            fc = f(c)?;
            fd = f(d)?;
        }
        let comparison = compare(&fc, &fd)?;
        let take_left = match mode {
            SearchMode::Minimize => comparison != Ordering::Greater,
            SearchMode::Maximize => comparison != Ordering::Less,
        };

        if take_left {
            hi = d;
            d = c;
            fd = fc;
            c = interpolate(lo, hi, INV_PHI2);
            fc = f(c)?;
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = interpolate(lo, hi, INV_PHI);
            fd = f(d)?;
        }
    }

    Ok((lo, hi))
}

/// Interpolate without overflowing when both finite endpoints have an infinite
/// difference, as happens for intervals spanning most of the f64 range.
#[inline]
fn interpolate(lo: f64, hi: f64, t: f64) -> f64 {
    let width = hi - lo;
    if width.is_finite() {
        lo + t * width
    } else {
        (1.0 - t) * lo + t * hi
    }
}
