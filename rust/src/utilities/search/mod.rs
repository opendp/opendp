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

/// Find the closest passing value to the decision boundary of `predicate`.
///
/// If `bounds` are not passed, conducts an exponential search to infer them first.
pub fn binary_search<T>(predicate: impl Fn(&T) -> bool, bounds: Option<(T, T)>) -> Fallible<T>
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
    bounds: Option<(T, T)>,
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
    bounds: Option<(T, T)>,
) -> Fallible<T>
where
    T: BinarySearchable,
{
    signed_fallible_binary_search(predicate, bounds).map(|(value, _sign)| value)
}

/// Fallible version of [`signed_binary_search`].
pub fn signed_fallible_binary_search<T>(
    predicate: impl Fn(&T) -> Fallible<bool>,
    bounds: Option<(T, T)>,
) -> Fallible<(T, i8)>
where
    T: BinarySearchable,
{
    let bounds = match bounds {
        Some(bounds) => bounds,
        None => fallible_exponential_bounds_search(&predicate)?
            .ok_or_else(|| err!(FailedFunction, "unable to infer bounds"))?,
    };
    signed_fallible_binary_search_with_bounds(predicate, bounds)
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
            FailedFunction,
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
                    let offset = match <$ty>::try_from(65_536_i128 * k as i128) {
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
                if sign < 0 {
                    return Vec::new();
                }

                let mut bands = vec![center];
                if let Some(next) = center.checked_add(1) {
                    bands.push(next);
                }

                for k in 1..=8 {
                    let offset = match <$ty>::try_from(65_536_u128 * k as u128) {
                        Ok(offset) => offset,
                        Err(_) => break,
                    };
                    let Some(candidate) = center.checked_add(offset) else {
                        break;
                    };
                    bands.push(candidate);
                }

                if bands.last() != Some(&<$ty>::MAX) {
                    bands.push(<$ty>::MAX);
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
            Ok(_) => return fallible!(FailedFunction, "predicate always fails"),
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
