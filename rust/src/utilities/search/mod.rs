use std::{
    mem::swap,
    ops::{Add, Div, Sub},
};

use num::{One, Zero};

use crate::error::Fallible;

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

macro_rules! impl_bands_signed_int {
    ($($ty:ty),+ $(,)?) => {
        $(impl Bands for $ty {
            fn bands(center: Self, sign: i8) -> Vec<Self> {
                let mut bands = vec![center];

                if sign > 0 {
                    if let Some(next) = center.checked_add(1) {
                        bands.push(next);
                    }
                } else if let Some(next) = center.checked_sub(1) {
                    bands.push(next);
                }

                for k in 1..=8 {
                    let offset = match <$ty>::try_from(2i128.pow(16) * k) {
                        Ok(offset) => offset,
                        Err(_) => break,
                    };
                    let candidate = if sign > 0 {
                        center.checked_add(offset)
                    } else {
                        center.checked_sub(offset)
                    };
                    let Some(candidate) = candidate else {
                        break;
                    };
                    bands.push(candidate);
                }

                let extreme = if sign > 0 { <$ty>::MAX } else { <$ty>::MIN };
                if bands.last() != Some(&extreme) {
                    bands.push(extreme);
                }

                bands
            }
        })+
    };
}
impl_bands_signed_int!(i8, i16, i32, i64, i128);

macro_rules! impl_bands_unsigned_int {
    ($($ty:ty),+ $(,)?) => {
        $(impl Bands for $ty {
            fn bands(center: Self, sign: i8) -> Vec<Self> {
                let mut bands = vec![center];

                if sign > 0 {
                    if let Some(next) = center.checked_add(1) {
                        bands.push(next);
                    }
                } else if let Some(next) = center.checked_sub(1) {
                    bands.push(next);
                }

                for k in 1..=8 {
                    let offset = match <$ty>::try_from(2i128.pow(16) * k) {
                        Ok(offset) => offset,
                        Err(_) => break,
                    };
                    let candidate = if sign > 0 {
                        center.checked_add(offset)
                    } else {
                        center.checked_sub(offset)
                    };
                    let Some(candidate) = candidate else {
                        break;
                    };
                    bands.push(candidate);
                }

                let extreme = if sign > 0 { <$ty>::MAX } else { <$ty>::MIN };
                if bands.last() != Some(&extreme) {
                    bands.push(extreme);
                }

                bands
            }
        })+
    };
}
impl_bands_unsigned_int!(u8, u16, u32, u64, u128);

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

const TERNARY_SEARCH_ITERS: usize = 160;

pub(crate) fn maximize_ternary(
    mut lo: f64,
    mut hi: f64,
    objective: impl Fn(f64) -> Fallible<f64>,
) -> Fallible<f64> {
    let mut best = objective(lo)?.max(objective(hi)?);

    for _ in 0..TERNARY_SEARCH_ITERS {
        let width = hi - lo;
        if width <= 0.0 {
            break;
        }

        let left = lo + width / 3.0;
        let right = hi - width / 3.0;
        if left == lo || right == hi || left >= right {
            break;
        }

        let left_value = objective(left)?;
        let right_value = objective(right)?;
        best = best.max(left_value).max(right_value);

        if left_value < right_value {
            lo = left;
        } else {
            hi = right;
        }
    }

    let mid = lo + (hi - lo) / 2.0;
    Ok(best.max(objective(mid)?))
}
