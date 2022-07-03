use std::{ops::{Div, Add, Sub}, mem::size_of};

use num::{One, Zero};

use crate::error::Fallible;



pub trait BinarySearchable: Halve {
    const TOLERANCE: Self;
}


macro_rules! impl_binary_searchable_float {
    ($($ty:ty),+) => ($(impl BinarySearchable for $ty {
        const TOLERANCE: Self = 0.;
    })+)
}
impl_binary_searchable_float!(f32, f64);

macro_rules! impl_binary_searchable_int {
    ($($ty:ty),+) => ($(impl BinarySearchable for $ty {
        const TOLERANCE: Self = 0;
    })+)
}
impl_binary_searchable_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

pub trait Halve {
    fn halve(&self) -> Self;
}
impl<T: One + Add<Output=T>> Halve for T
    where for<'a> &'a T: Div<T, Output=T> {
    fn halve(&self) -> Self {
        // TODO: typenum traits to avoid this
        self / (T::one() + T::one())
    }
}

pub fn binary_search<T>(
    predicate: impl Fn(&T) -> bool, bounds: (T, T)
) -> Fallible<T>
    where T: Clone + Bands + Zero + PartialEq + Add<Output=T> + Sub<Output=T> + PartialOrd + Clone + BinarySearchable {
    signed_binary_search(predicate, bounds).map(|v| v.0)
}

pub fn signed_binary_search<T>(
    predicate: impl Fn(&T) -> bool, bounds: (T, T)
) -> Fallible<(T, bool)> 
    where T: Clone + Bands + Zero + PartialEq + Add<Output=T> + Sub<Output=T> + PartialOrd + Clone + BinarySearchable {
    let predicate = move |v: &T| Ok(predicate(v));
    signed_fallible_binary_search(predicate, bounds)
}

pub fn fallible_binary_search<T>(
    predicate: impl Fn(&T) -> Fallible<bool>, bounds: (T, T)
) -> Fallible<T> 
    where T: Clone + Bands + Zero + PartialEq + Add<Output=T> + Sub<Output=T> + PartialOrd + Clone + BinarySearchable {
    signed_fallible_binary_search(predicate, bounds).map(|v| v.0)
}

pub fn signed_fallible_binary_search<T>(
    predicate: impl Fn(&T) -> Fallible<bool>, bounds: (T, T)
) -> Fallible<(T, bool)>
    where T: Clone + Bands + Zero + PartialEq + Add<Output=T> + Sub<Output=T> + PartialOrd + Clone + BinarySearchable {

    let (mut lower, mut upper) = bounds;
    if lower > upper {
        return fallible!(FailedFunction, "lower may not be greater than upper")
    }

    let minimize = predicate(&lower)?;
    let maximize = predicate(&upper)?;

    if maximize == minimize {
        return fallible!(FailedFunction, "the decision boundary of the predicate is outside the bounds")
    }

    let mut mid = lower.clone();

    while upper.clone() - lower.clone() > T::TOLERANCE {
        let new_mid = lower.clone() + (upper.clone() - lower.clone()).halve();  // avoid overflow

        // avoid an infinite loop from float roundoff
        if new_mid == mid { break }

        mid = new_mid;
        if predicate(&mid)? == minimize {
            upper = mid.clone();
        } else {
            lower = mid.clone();
        }
    }
    // one bound is always false, the other true. Return the truthy bound
    Ok((if minimize { upper } else { lower }, minimize))
}

pub trait Bands: Sized {
    fn bands(center: Self, positive: bool) -> Vec<Self>;
}

macro_rules! impl_bands_float {
    ($($ty:ty),+) => ($(impl Bands for $ty  {
        /// searching bands of [2^((k - 1)^2), 2^(k^2)].
        /// exponent has ten bits (2.^1024 overflows) so k must be in [0, 32).
        /// unlikely to need numbers greater than 2**64, and to avoid overflow from shifted centers,
        ///    only check k in [0, 8). Set your own bounds if this is not sufficient
        /// 
        /// Similarly for f32, search for k in [0, 4)
        fn bands(center: Self, positive: bool) -> Vec<Self> {
            let f_sign = if positive {1.} else {-1.};
            let _2: Self = 2.0;
            let iter = vec!(center).into_iter()
                .chain((0..size_of::<Self>())
                    .map(|k| center + f_sign * _2.powi((k as i32).pow(2))));
            if positive {iter.collect()} else {iter.rev().collect()}
        }
    })+)
}
impl_bands_float!(f32, f64);

macro_rules! impl_bands_signed_int {
    ($($ty:ty),+) => ($(impl Bands for $ty {
        /// searching bands of [(k - 1) * 2^16, k * 2^16].
        /// center + 1 included because zero is prone to error
        fn bands(center: Self, positive: bool) -> Vec<Self> {
            let i_sign = if positive {1} else {-1};
            let iter = vec!(center, center + 1).into_iter()
                .chain((1..9).map(|k| center + i_sign * 2 << (size_of::<Self>() as Self * 2) * k));
            if positive {iter.collect()} else {iter.rev().collect()}
        }
    })+)
}

impl_bands_signed_int!(i8, i16, i32, i64, i128);

macro_rules! impl_bands_unsigned_int {
    ($($ty:ty),+) => ($(impl Bands for $ty {
        /// searching bands of [(k - 1) * 2^16, k * 2^16].
        /// center + 1 included because zero is prone to error
        fn bands(center: Self, positive: bool) -> Vec<Self> {
            if !positive  { return Vec::new() }
            let iter = vec!(center, center + 1).into_iter()
                .chain((1..9).map(|k| center + 2 << (size_of::<Self>() as Self * 2) * k));
            if positive {iter.collect()} else {iter.rev().collect()}
        }
    })+)
}
impl_bands_unsigned_int!(u8, u16, u32, u64, u128);

pub fn exponential_bounds_search<T>(
    predicate: &impl Fn(&T) -> bool,
) -> Option<(T, T)>
    where T: Bands + Zero + PartialEq + Add<Output=T> + Sub<Output=T> + PartialOrd + Clone + BinarySearchable {

    // identify which band (of eight) the decision boundary lies in, 
    // starting from `center` in the direction indicated by `sign`
    let signed_band_search = |center: T, at_center: bool, sign: bool| -> Option<(T, T)> {
        let bands = T::bands(center, sign);

        for window in bands.windows(2) {
            // looking for a change in sign that indicates the decision boundary is within this band
            if at_center != predicate(&window[1]) {
                // return the band
                return Some((window[0].clone(), window[1].clone()))
            }
        }
        // No band found!
        None
    };

    let center = T::zero();
    let at_center = predicate(&center);

    signed_band_search(center.clone(), at_center, true)
        .or(signed_band_search(center, at_center, false))
}


/// Determine bounds for a binary search via an exponential search,
/// in large bands of [2^((k - 1)^2), 2^(k^2)] for k in [0, 8).
/// Will attempt to recover once if `predicate` throws an exception, 
/// by searching bands on the ok side of the exception boundary.
/// 
/// :param predicate: a monotonic unary function from a number to a boolean
/// :param T: type of argument to predicate, one of {float, int}
/// :return: a tuple of float or int bounds that the decision boundary lies within
/// :raises TypeError: if the type is not inferrable (pass T)
/// :raises ValueError: if the predicate function is constant
pub fn fallible_exponential_bounds_search<T>(
    predicate: &impl Fn(&T) -> Fallible<bool>
) -> Fallible<Option<(T, T)>>
    where T: Bands + Zero + PartialEq + Add<Output=T> + Sub<Output=T> + PartialOrd + Clone + BinarySearchable {

    // identify which band (of eight) the decision boundary lies in, 
    // starting from `center` in the direction indicated by `sign`
    let signed_band_search = |center: T, at_center: bool, sign: bool| -> Option<(T, T)> {
        let bands = T::bands(center, sign);

        for window in bands.windows(2) {
            // looking for a change in sign that indicates the decision boundary is within this band
            if at_center != predicate(&window[1]).ok()? {
                // return the band
                return Some((window[0].clone(), window[1].clone()))
            }
        }
        
        // No band found!
        None
    };

    let center = T::zero();
    if let Ok(at_center) = predicate(&center) {
        return Ok(signed_band_search(center.clone(), at_center, true)
            .or(signed_band_search(center, at_center, false)))
    }

    // predicate has thrown an exception
    // 1. Treat exceptions as a secondary decision boundary, and find the edge value
    // 2. Return a bound by searching from the exception edge, in the direction away from the exception
    let exception_predicate = |v: &T| predicate(v).is_ok();

    let exception_bounds = exponential_bounds_search(&exception_predicate)
        .ok_or_else(|| err!(FailedFunction, "predicate always fails"))?;
    
    let (center, sign) = signed_binary_search(exception_predicate, exception_bounds)?;
    let at_center = predicate(&center)?;
    Ok(signed_band_search(center, at_center, sign))
}
