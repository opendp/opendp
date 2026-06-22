use std::{
    iter::{once, successors},
    mem::swap,
    ops::{Add, Div, Sub},
};

use num::{CheckedAdd, CheckedSub, One, Zero};

use crate::{
    error::Fallible,
    traits::{ExactIntCast, FiniteBounds},
};

#[cfg(test)]
mod test;

/// Types that support OpenDP's binary-search utilities.
pub trait BinarySearchable:
    Bands + Zero + One + Clone + PartialEq + PartialOrd + Add<Output = Self> + Sub<Output = Self>
{
    fn midpoint(lower: &Self, upper: &Self) -> Self;
}

macro_rules! impl_binary_searchable_float {
    ($($ty:ty),+ $(,)?) => {
        $(impl BinarySearchable for $ty {
            fn midpoint(lower: &Self, upper: &Self) -> Self {
                lower + (upper - lower).halve()
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
    signed_fallible_binary_search_with_bounds(predicate, bounds)
}

fn resolve_bounds<T>(
    predicate: &impl Fn(&T) -> Fallible<bool>,
    bounds: impl BoundSpec<T>,
) -> Fallible<(T, T)>
where
    T: BinarySearchable,
{
    match bounds.resolve() {
        (Some(lower), Some(upper)) => Ok((lower, upper)),
        (Some(lower), None) => {
            let at_lower = predicate(&lower)?;
            fallible_signed_band_search(predicate, lower.clone(), at_lower, 1)?
                .ok_or_else(|| err!(Search, "unable to infer upper bound"))
        }
        (None, Some(upper)) => {
            let at_upper = predicate(&upper)?;
            fallible_signed_band_search(predicate, upper.clone(), at_upper, -1)?
                .ok_or_else(|| err!(Search, "unable to infer lower bound"))
        }
        (None, None) => fallible_exponential_bounds_search(predicate)?
            .ok_or_else(|| err!(Search, "unable to infer bounds")),
    }
}

fn signed_fallible_binary_search_with_bounds<T>(
    predicate: impl Fn(&T) -> Fallible<bool>,
    bounds: (T, T),
) -> Fallible<(T, i8)>
where
    T: BinarySearchable,
{
    let (mut lower, mut upper) = bounds;
    if lower > upper {
        swap(&mut lower, &mut upper);
    }

    let maximize = predicate(&lower)?;
    let minimize = predicate(&upper)?;

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
                let half: Self = 0.5;
                let two: Self = 2.0;

                let mut bands = vec![center, center + sign * half];
                bands.extend(
                    (0..std::mem::size_of::<Self>())
                        .map(|k| center + sign * two.powi((k as i32).pow(2))),
                );
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

pub fn exponential_bounds_search<T>(predicate: &impl Fn(&T) -> bool) -> Option<(T, T)>
where
    T: BinarySearchable,
{
    let center = T::zero();
    let at_center = predicate(&center);

    signed_band_search(predicate, center.clone(), at_center, 1)
        .or_else(|| signed_band_search(predicate, center, at_center, -1))
}

/// Determine bounds for a binary search via an exponential search.
///
/// If `predicate` fails at the origin, recover by first finding the edge of the exceptional region
/// and then searching away from it.
pub fn fallible_exponential_bounds_search<T>(
    predicate: &impl Fn(&T) -> Fallible<bool>,
) -> Fallible<Option<(T, T)>>
where
    T: BinarySearchable,
{
    let center = T::zero();
    let center_result = predicate(&center);

    if let Ok(at_center) = center_result.as_ref() {
        match fallible_signed_band_search(predicate, center.clone(), *at_center, 1) {
            Ok(Some(bounds)) => return Ok(Some(bounds)),
            Ok(None) => return fallible_signed_band_search(predicate, center, *at_center, -1),
            Err(_) => {}
        }
    }

    let exception_predicate = |value: &T| predicate(value).is_ok();

    let exception_bounds = match exponential_bounds_search(&exception_predicate) {
        Some(bounds) => bounds,
        None => match center_result {
            Ok(_) => return fallible!(Search, "predicate always fails"),
            Err(err) => return Err(err),
        },
    };

    let (center, sign) = signed_fallible_binary_search_with_bounds(
        |value| Ok(exception_predicate(value)),
        exception_bounds,
    )?;
    let at_center = predicate(&center)?;
    fallible_signed_band_search(predicate, center, at_center, sign)
}

fn signed_band_search<T>(
    predicate: &impl Fn(&T) -> bool,
    center: T,
    at_center: bool,
    sign: i8,
) -> Option<(T, T)>
where
    T: BinarySearchable,
{
    let bands = T::bands(center, sign);

    for window in bands.windows(2) {
        if at_center != predicate(&window[1]) {
            let mut lower = window[0].clone();
            let mut upper = window[1].clone();
            if lower > upper {
                swap(&mut lower, &mut upper);
            }
            return Some((lower, upper));
        }
    }

    None
}

fn fallible_signed_band_search<T>(
    predicate: &impl Fn(&T) -> Fallible<bool>,
    center: T,
    at_center: bool,
    sign: i8,
) -> Fallible<Option<(T, T)>>
where
    T: BinarySearchable,
{
    let bands = T::bands(center, sign);

    for window in bands.windows(2) {
        if at_center != predicate(&window[1])? {
            let mut lower = window[0].clone();
            let mut upper = window[1].clone();
            if lower > upper {
                swap(&mut lower, &mut upper);
            }
            return Ok(Some((lower, upper)));
        }
    }

    Ok(None)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchMode {
    Minimize,
    Maximize,
}

impl SearchMode {
    #[inline]
    pub(crate) fn bad_value(self) -> f64 {
        match self {
            SearchMode::Minimize => f64::INFINITY,
            SearchMode::Maximize => f64::NEG_INFINITY,
        }
    }

    #[inline]
    pub(crate) fn sanitize(self, value: f64) -> f64 {
        if value.is_nan() {
            self.bad_value()
        } else {
            value
        }
    }

    #[inline]
    pub(crate) fn is_better(self, candidate: f64, incumbent: f64) -> bool {
        match self {
            SearchMode::Minimize => candidate < incumbent,
            SearchMode::Maximize => candidate > incumbent,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Optimum {
    pub(crate) arg: f64,
    pub(crate) value: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BracketedOptimum {
    /// Best sampled/refined argument found by the search.
    pub(crate) arg: f64,

    /// Objective value at `arg`, sanitized according to the search mode.
    pub(crate) value: f64,

    /// Final search bracket containing `arg`.
    ///
    /// For a unimodal objective, this is the final golden-section bracket.
    /// For grid-plus-local-refinement, this is the refined bracket for the
    /// best local extremum found by the grid.
    pub(crate) lo: f64,
    pub(crate) hi: f64,
}

impl BracketedOptimum {
    #[inline]
    pub(crate) fn optimum(self) -> Optimum {
        Optimum {
            arg: self.arg,
            value: self.value,
        }
    }
}

/// Optimize a scalar objective over `[lo, hi]` to floating-point precision.
///
/// When `local_grid` is `None` or less than 3, this assumes the objective is
/// unimodal and performs one golden-section search. When `Some(n)`, it samples
/// `n` grid points and refines every local extremum. This is useful for
/// envelopes/minima that may have kinks.
#[inline]
pub(crate) fn optimize_to_precision<F>(
    mode: SearchMode,
    lo: f64,
    hi: f64,
    local_grid: Option<usize>,
    f: F,
) -> Optimum
where
    F: Fn(f64) -> f64,
{
    optimize_to_precision_bracket(mode, lo, hi, local_grid, f).optimum()
}

/// Like [`optimize_to_precision`], but also returns the final small bracket.
#[inline]
pub(crate) fn optimize_to_precision_bracket<F>(
    mode: SearchMode,
    lo: f64,
    hi: f64,
    local_grid: Option<usize>,
    f: F,
) -> BracketedOptimum
where
    F: Fn(f64) -> f64,
{
    if !(hi > lo) || !lo.is_finite() || !hi.is_finite() {
        let value = mode.sanitize(f(lo));
        return BracketedOptimum {
            arg: lo,
            value,
            lo,
            hi: lo,
        };
    }

    let grid = local_grid.unwrap_or(0);

    if grid < 3 {
        return golden_search_to_precision(mode, lo, hi, &f);
    }

    let grid = grid.max(3);
    let mut xs = Vec::with_capacity(grid);
    let mut vals = Vec::with_capacity(grid);

    let mut best = BracketedOptimum {
        arg: lo,
        value: mode.sanitize(f(lo)),
        lo,
        hi: lo,
    };

    for i in 0..grid {
        let t = i as f64 / (grid - 1) as f64;
        let x = lo + t * (hi - lo);
        let value = mode.sanitize(f(x));

        xs.push(x);
        vals.push(value);

        if mode.is_better(value, best.value) {
            best = BracketedOptimum {
                arg: x,
                value,
                lo: x,
                hi: x,
            };
        }
    }

    for i in 0..grid {
        let is_local_extremum = match mode {
            SearchMode::Minimize => {
                (i == 0 || vals[i] <= vals[i - 1]) && (i + 1 == grid || vals[i] <= vals[i + 1])
            }
            SearchMode::Maximize => {
                (i == 0 || vals[i] >= vals[i - 1]) && (i + 1 == grid || vals[i] >= vals[i + 1])
            }
        };

        if !is_local_extremum {
            continue;
        }

        let left = if i == 0 { xs[i] } else { xs[i - 1] };
        let right = if i + 1 == grid { xs[i] } else { xs[i + 1] };

        if right > left {
            let candidate = golden_search_to_precision(mode, left, right, &f);
            if mode.is_better(candidate.value, best.value) {
                best = candidate;
            }
        }
    }

    best
}

/// Optimize an objective over positive arguments using `x = ln(arg)`.
#[inline]
pub(crate) fn optimize_log_domain_to_precision<F>(
    mode: SearchMode,
    arg_lo: f64,
    arg_hi: f64,
    local_grid: Option<usize>,
    f_arg: F,
) -> Optimum
where
    F: Fn(f64) -> f64,
{
    optimize_log_domain_to_precision_bracket(mode, arg_lo, arg_hi, local_grid, f_arg).optimum()
}

/// Like [`optimize_log_domain_to_precision`], but also returns a bracket in the
/// original positive argument domain.
#[inline]
pub(crate) fn optimize_log_domain_to_precision_bracket<F>(
    mode: SearchMode,
    arg_lo: f64,
    arg_hi: f64,
    local_grid: Option<usize>,
    f_arg: F,
) -> BracketedOptimum
where
    F: Fn(f64) -> f64,
{
    if !(arg_lo > 0.0) || !(arg_hi > arg_lo) || !arg_lo.is_finite() || !arg_hi.is_finite() {
        let value = mode.sanitize(f_arg(arg_lo));
        return BracketedOptimum {
            arg: arg_lo,
            value,
            lo: arg_lo,
            hi: arg_lo,
        };
    }

    let x_lo = arg_lo.ln();
    let x_hi = arg_hi.ln();

    let f_x = |x: f64| -> f64 {
        let arg = x.exp();
        if !(arg > 0.0) || !arg.is_finite() {
            return mode.bad_value();
        }
        mode.sanitize(f_arg(arg))
    };

    let optimum = optimize_to_precision_bracket(mode, x_lo, x_hi, local_grid, f_x);

    BracketedOptimum {
        arg: optimum.arg.exp().clamp(arg_lo, arg_hi),
        value: optimum.value,
        lo: optimum.lo.exp().clamp(arg_lo, arg_hi),
        hi: optimum.hi.exp().clamp(arg_lo, arg_hi),
    }
}

/// Sample a positive interval on a log-spaced grid without refinement.
///
/// This is intended for cheap cap-finding passes where final tightness is not
/// determined by the probe itself.
#[inline]
pub(crate) fn sample_log_domain<F>(
    mode: SearchMode,
    arg_lo: f64,
    arg_hi: f64,
    grid: usize,
    f_arg: F,
) -> Optimum
where
    F: Fn(f64) -> f64,
{
    if !(arg_lo > 0.0) || !(arg_hi > arg_lo) || !arg_lo.is_finite() || !arg_hi.is_finite() {
        return Optimum {
            arg: arg_lo,
            value: mode.sanitize(f_arg(arg_lo)),
        };
    }

    let grid = grid.max(2);
    let x_lo = arg_lo.ln();
    let x_hi = arg_hi.ln();

    let mut best = Optimum {
        arg: arg_lo,
        value: mode.sanitize(f_arg(arg_lo)),
    };

    for i in 0..grid {
        let t = i as f64 / (grid - 1) as f64;
        let arg = (x_lo + t * (x_hi - x_lo)).exp();
        let value = mode.sanitize(f_arg(arg));

        if mode.is_better(value, best.value) {
            best = Optimum { arg, value };
        }
    }

    best
}

fn golden_search_to_precision<F>(
    mode: SearchMode,
    mut lo: f64,
    mut hi: f64,
    f: &F,
) -> BracketedOptimum
where
    F: Fn(f64) -> f64,
{
    const INV_PHI: f64 = 0.6180339887498949;
    const INV_PHI2: f64 = 0.3819660112501051;

    if !(hi > lo) {
        let value = mode.sanitize(f(lo));
        return BracketedOptimum {
            arg: lo,
            value,
            lo,
            hi: lo,
        };
    }

    let mut c = lo + INV_PHI2 * (hi - lo);
    let mut d = lo + INV_PHI * (hi - lo);

    let mut fc = mode.sanitize(f(c));
    let mut fd = mode.sanitize(f(d));

    loop {
        let old_lo = lo;
        let old_hi = hi;

        let take_left = match mode {
            SearchMode::Minimize => fc <= fd,
            SearchMode::Maximize => fc >= fd,
        };

        if take_left {
            hi = d;
            d = c;
            fd = fc;
            c = lo + INV_PHI2 * (hi - lo);
            fc = mode.sanitize(f(c));
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + INV_PHI * (hi - lo);
            fd = mode.sanitize(f(d));
        }

        if lo == old_lo && hi == old_hi {
            break;
        }

        if c == lo || c == hi || d == lo || d == hi {
            break;
        }
    }

    let mut best = BracketedOptimum {
        arg: lo,
        value: mode.sanitize(f(lo)),
        lo,
        hi,
    };

    for candidate in [
        Optimum {
            arg: hi,
            value: mode.sanitize(f(hi)),
        },
        Optimum { arg: c, value: fc },
        Optimum { arg: d, value: fd },
    ] {
        if mode.is_better(candidate.value, best.value) {
            best.arg = candidate.arg;
            best.value = candidate.value;
        }
    }

    best
}
