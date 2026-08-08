use std::{
    cmp::Ordering,
    iter::{once, successors},
    mem::swap,
    ops::{Add, Div, Sub},
};

use num::{CheckedAdd, CheckedSub, One, Zero};

use crate::{
    error::{ErrorVariant, Fallible},
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
                if lower.is_sign_negative() != upper.is_sign_negative() {
                    lower / 2.0 + upper / 2.0
                } else {
                    lower + (upper - lower).halve()
                }
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

/// Find the boundary of a monotone comparator.
///
/// The callback compares its argument to the target. Range variants are
/// consumed here as ordering information: `NumericRangeBelow` means `Less`,
/// and `NumericRangeAbove` means `Greater`. A callback may return a range
/// variant only when that side describes the final quantity being compared to
/// the target; operation-local range failures must remain ordinary errors.
///
/// If the comparator returns `Equal`, that value is returned. If no exact
/// value exists, the endpoint whose comparison is `Less` is returned. Thus an
/// increasing comparator returns the lower bracket, while a decreasing
/// comparator returns the upper bracket.
pub fn fallible_binary_search_by<T>(
    comparison: impl Fn(&T) -> Fallible<Ordering>,
    bounds: impl BoundSpec<T>,
) -> Fallible<T>
where
    T: BinarySearchable,
{
    signed_fallible_binary_search_by(comparison, bounds).map(|(value, _sign)| value)
}

fn signed_fallible_binary_search_by<T>(
    comparison: impl Fn(&T) -> Fallible<Ordering>,
    bounds: impl BoundSpec<T>,
) -> Fallible<(T, i8)>
where
    T: BinarySearchable,
{
    let bounds = resolve_comparison_bounds(&comparison, bounds)?;
    signed_fallible_binary_search_by_with_bounds(
        &comparison,
        bounds,
        "the comparator does not cross the target within the bounds",
    )
}

fn ordered_result(result: Fallible<Ordering>) -> Fallible<Ordering> {
    match result {
        Ok(ordering) => Ok(ordering),
        Err(error) if error.variant == ErrorVariant::NumericRangeBelow => Ok(Ordering::Less),
        Err(error) if error.variant == ErrorVariant::NumericRangeAbove => Ok(Ordering::Greater),
        Err(error) => Err(error),
    }
}

fn resolve_comparison_bounds<T>(
    comparison: &impl Fn(&T) -> Fallible<Ordering>,
    bounds: impl BoundSpec<T>,
) -> Fallible<(T, T)>
where
    T: BinarySearchable,
{
    let comparison = |value: &T| ordered_result(comparison(value));
    match bounds.resolve() {
        (Some(lower), Some(upper)) => Ok((lower, upper)),
        (Some(lower), None) => {
            let at_lower = comparison(&lower)?;
            if at_lower == Ordering::Equal {
                return Ok((lower.clone(), lower));
            }
            fallible_signed_band_search_by(&comparison, lower.clone(), at_lower, 1)?.ok_or_else(
                || {
                    err!(
                        Search,
                        "the decision boundary is below the lower bound or the comparator does not change above it"
                    )
                },
            )
        }
        (None, Some(upper)) => {
            let at_upper = comparison(&upper)?;
            if at_upper == Ordering::Equal {
                return Ok((upper.clone(), upper));
            }
            fallible_signed_band_search_by(&comparison, upper.clone(), at_upper, -1)?.ok_or_else(
                || {
                    err!(
                        Search,
                        "the decision boundary is above the upper bound or the comparator does not change below it"
                    )
                },
            )
        }
        (None, None) => {
            let center = T::zero();
            let at_center = comparison(&center)?;
            if at_center == Ordering::Equal {
                return Ok((center.clone(), center));
            }
            fallible_exponential_bounds_search_by(&comparison, center, at_center)?
                .ok_or_else(|| err!(Search, "unable to infer bounds for comparator"))
        }
    }
}

fn signed_fallible_binary_search_by_with_bounds<T>(
    comparison: &impl Fn(&T) -> Fallible<Ordering>,
    bounds: (T, T),
    boundary_error: &'static str,
) -> Fallible<(T, i8)>
where
    T: BinarySearchable,
{
    let (mut lower, mut upper) = bounds;
    if lower > upper {
        swap(&mut lower, &mut upper);
    }

    let lower_order = ordered_result(comparison(&lower))?;
    let upper_order = ordered_result(comparison(&upper))?;
    if lower_order == Ordering::Equal {
        return Ok((lower, 0));
    }
    if upper_order == Ordering::Equal {
        return Ok((upper, 0));
    }
    if !matches!(
        (lower_order, upper_order),
        (Ordering::Less, Ordering::Greater) | (Ordering::Greater, Ordering::Less)
    ) {
        return fallible!(Search, "{boundary_error}");
    }

    let mut mid = lower.clone();
    loop {
        let new_mid = T::midpoint(&lower, &upper);
        if new_mid == mid || new_mid == lower || new_mid == upper {
            break;
        }

        mid = new_mid;
        match ordered_result(comparison(&mid))? {
            Ordering::Equal => return Ok((mid, 0)),
            ordering if ordering == lower_order => lower = mid.clone(),
            ordering if ordering == upper_order => upper = mid.clone(),
            _ => {
                return fallible!(Search, "the comparator is not monotone within the bounds");
            }
        }
    }

    Ok(if lower_order == Ordering::Less {
        (lower, -1)
    } else {
        (upper, 1)
    })
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
    let comparison = |value: &T| {
        predicate(value).map(|passes| {
            if passes {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    };
    signed_fallible_binary_search_by_with_bounds(
        &comparison,
        bounds,
        "the decision boundary of the predicate is outside the bounds",
    )
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
                .ok_or_else(|| {
                    err!(
                        Search,
                        "the decision boundary is below the lower bound or the predicate does not change above it"
                    )
                })
        }
        (None, Some(upper)) => {
            let at_upper = predicate(&upper)?;
            fallible_signed_band_search(predicate, upper.clone(), at_upper, -1)?
                .ok_or_else(|| {
                    err!(
                        Search,
                        "the decision boundary is above the upper bound or the predicate does not change below it"
                    )
                })
        }
        (None, None) => fallible_exponential_bounds_search(predicate)?
            .ok_or_else(|| err!(Search, "unable to infer bounds")),
    }
}

pub trait Bands: Sized {
    fn bands(center: Self, sign: i8) -> Vec<Self>;
}

macro_rules! impl_bands_float {
    ($($ty:ty),+ $(,)?) => {
        $(impl Bands for $ty {
            fn bands(center: Self, sign: i8) -> Vec<Self> {
                let sign_value = sign;
                let sign: Self = if sign_value > 0 { 1.0 } else { -1.0 };
                let half: Self = 0.5;
                let two: Self = 2.0;

                let mut bands = vec![center];
                let first = center + sign * half;
                if first.is_finite() {
                    bands.push(first);
                }
                for k in 0..std::mem::size_of::<Self>() {
                    let candidate = center + sign * two.powi((k as i32).pow(2));
                    if candidate.is_finite() {
                        bands.push(candidate);
                    } else {
                        break;
                    }
                }
                let extreme = if sign_value > 0 { <$ty>::MAX } else { <$ty>::MIN };
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
///
/// Integer searches use exponentially increasing bands. Floating-point searches also include the
/// finite type extrema, so they do not stop at the old `2^(k^2)` sequence.
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
/// Integer searches use exponentially increasing bands. Floating-point searches also include the
/// finite type extrema, so they do not stop at the old `2^(k^2)` sequence. If `predicate` fails at
/// the origin, recover by first finding the edge of the exceptional region and then searching away
/// from it.
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

    let comparison = |value: &T| {
        Ok(if exception_predicate(value) {
            Ordering::Less
        } else {
            Ordering::Greater
        })
    };
    let (center, sign) = signed_fallible_binary_search_by_with_bounds(
        &comparison,
        exception_bounds,
        "the decision boundary of the predicate is outside the bounds",
    )?;
    let at_center = predicate(&center)?;
    fallible_signed_band_search(predicate, center, at_center, sign)
}

fn fallible_exponential_bounds_search_by<T, V>(
    evaluator: &impl Fn(&T) -> Fallible<V>,
    center: T,
    at_center: V,
) -> Fallible<Option<(T, T)>>
where
    T: BinarySearchable,
    V: Clone + PartialEq,
{
    if let Some(bounds) =
        fallible_signed_band_search_by(evaluator, center.clone(), at_center.clone(), 1)?
    {
        return Ok(Some(bounds));
    }
    fallible_signed_band_search_by(evaluator, center, at_center, -1)
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
    fallible_signed_band_search_by(predicate, center, at_center, sign)
}

fn fallible_signed_band_search_by<T, V>(
    evaluator: &impl Fn(&T) -> Fallible<V>,
    center: T,
    at_center: V,
    sign: i8,
) -> Fallible<Option<(T, T)>>
where
    T: BinarySearchable,
    V: PartialEq,
{
    let bands = T::bands(center, sign);

    for window in bands.windows(2) {
        if at_center != evaluator(&window[1])? {
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
