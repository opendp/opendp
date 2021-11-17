use std::cmp::Ordering;
use std::ops::{BitAnd, Shl, Shr, Sub};

use num::{One, Zero};

use crate::error::Fallible;

pub trait CheckNull { fn is_null(&self) -> bool; }

/// TotalOrd is well-defined on types that are Ord on their non-null values.
/// The framework provides a way to ensure values are non-null at runtime.
/// This trait should only be used when the framework can rely on these assurances.
/// TotalOrd shares the same interface as Ord, but with a total_ prefix on methods
pub trait TotalOrd: PartialOrd + Sized {
    fn total_cmp(&self, other: &Self) -> Fallible<Ordering>;
    fn total_max(self, other: Self) -> Fallible<Self> { max_by(self, other, TotalOrd::total_cmp) }
    fn total_min(self, other: Self) -> Fallible<Self> { min_by(self, other, TotalOrd::total_cmp) }
    fn total_clamp(self, min: Self, max: Self) -> Fallible<Self> {
        if min > max { return fallible!(FailedFunction, "min cannot be greater than max") }
        Ok(if let Ordering::Less = self.total_cmp(&min)? {
            min
        } else if let Ordering::Greater = self.total_cmp(&max)? {
            max
        } else {
            self
        })
    }
}


pub trait FloatBits: Sized {
    type Bits: Copy + One + Zero + Eq
    + Shr<Output=Self::Bits> + Shl<Output=Self::Bits>
    + BitAnd<Output=Self::Bits> + Sub<Output=Self::Bits>;
    // Number of bits in exponent
    const EXPONENT_BITS: Self::Bits;
    // Number of bits in mantissa, equal to Self::MANTISSA_DIGITS - 1
    const MANTISSA_BITS: Self::Bits;
    // Greatest number of bits set in exponent when bounded above by 1
    const EXPONENT_PROB: Self::Bits;

    fn sign(self) -> bool {
        (self.to_bits() & (Self::Bits::one() << (Self::EXPONENT_BITS + Self::MANTISSA_BITS))) == Self::Bits::zero()
    }
    fn exponent(self) -> Self::Bits {
        (self.to_bits() >> Self::MANTISSA_BITS) & ((Self::Bits::one() << Self::EXPONENT_BITS) - Self::Bits::one())
    }
    fn mantissa(self) -> Self::Bits {
        self.to_bits() & ((Self::Bits::one() << Self::MANTISSA_BITS) - Self::Bits::one())
    }
    fn to_bits(self) -> Self::Bits;
}


macro_rules! impl_check_null_for_non_nullable {
    ($($ty:ty),+) => {
        $(impl CheckNull for $ty {
            #[inline]
            fn is_null(&self) -> bool {false}
        })+
    }
}
impl_check_null_for_non_nullable!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool, String, &str, char, usize, isize);
impl<T: CheckNull> CheckNull for Option<T> {
    #[inline]
    fn is_null(&self) -> bool {
        if let Some(v) = self {
            v.is_null()
        } else { true }
    }
}
macro_rules! impl_check_null_for_float {
    ($($ty:ty),+) => {
        $(impl CheckNull for $ty {
            #[inline]
            fn is_null(&self) -> bool {self.is_nan()}
        })+
    }
}
impl_check_null_for_float!(f64, f32);

// TRAIT TotalOrd
macro_rules! impl_total_ord_for_ord {
    ($($ty:ty),*) => {$(impl TotalOrd for $ty {
        fn total_cmp(&self, other: &Self) -> Fallible<Ordering> {Ok(Ord::cmp(self, other))}
    })*}
}
impl_total_ord_for_ord!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize);

macro_rules! impl_total_ord_for_float {
    ($($ty:ty),*) => {
        $(impl TotalOrd for $ty {
            fn total_cmp(&self, other: &Self) -> Fallible<Ordering> {
                PartialOrd::partial_cmp(self, other)
                    .ok_or_else(|| err!(FailedFunction, concat!(stringify!($ty), " cannot not be null when clamping.")))
            }
        })*
    }
}
impl_total_ord_for_float!(f64, f32);

pub fn max_by<T, F: FnOnce(&T, &T) -> Fallible<Ordering>>(v1: T, v2: T, compare: F) -> Fallible<T> {
    compare(&v1, &v2).map(|cmp| match cmp {
        Ordering::Less | Ordering::Equal => v2,
        Ordering::Greater => v1,
    })
}

pub fn min_by<T, F: FnOnce(&T, &T) -> Fallible<Ordering>>(v1: T, v2: T, compare: F) -> Fallible<T> {
    compare(&v1, &v2).map(|cmp| match cmp {
        Ordering::Less | Ordering::Equal => v1,
        Ordering::Greater => v2,
    })
}

impl<T1: TotalOrd, T2: TotalOrd> TotalOrd for (T1, T2) {
    fn total_cmp(&self, other: &Self) -> Fallible<Ordering> {
        let cmp = self.0.total_cmp(&other.0)?;
        if Ordering::Equal == cmp {
            self.1.total_cmp(&other.1)
        } else {
            Ok(cmp)
        }
    }
}


// TRAIT FloatBits
impl FloatBits for f64 {
    type Bits = u64;
    const EXPONENT_BITS: u64 = 11;
    const MANTISSA_BITS: u64 = 52;
    // 2^(EXPONENT_BITS - 1) - 2
    //     subtract one to exclude all numbers >= 1f64
    //     subtract one for zero indexing
    const EXPONENT_PROB: u64 = 1022;
    fn to_bits(self) -> Self::Bits { self.to_bits() }
}

impl FloatBits for f32 {
    type Bits = u32;
    const EXPONENT_BITS: u32 = 8;
    const MANTISSA_BITS: u32 = 23;
    // 2^(EXPONENT_BITS - 1) - 2
    const EXPONENT_PROB: u32 = 126;
    fn to_bits(self) -> Self::Bits { self.to_bits() }
}
