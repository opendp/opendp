use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{BitAnd, BitOr, Shl, Shr, Sub};

use dashu::integer::IBig;
use dashu::rational::RBig;
use num::{One, Zero};

use crate::domains::Bounds;
use crate::error::Fallible;
#[cfg(feature = "contrib")]
use crate::interactive::Queryable;

use super::ExactIntCast;

/// Returns the length of self, where self is a collection.
///
/// Self is commonly a `Vec` or `HashMap`.
pub trait CollectionSize {
    /// # Proof Definition
    /// For any `value` of type `Self`, returns the size of the collection.
    fn size(&self) -> usize;
}

pub trait CheckAtom: CheckNull + Sized + Clone + PartialEq + Debug + Send + Sync {
    fn is_bounded(&self, _bounds: Bounds<Self>) -> Fallible<bool> {
        fallible!(FailedFunction, "bounds check is not implemented")
    }
    fn check_member(&self, bounds: Option<Bounds<Self>>, nullable: bool) -> Fallible<bool> {
        if let Some(bounds) = bounds {
            if !self.is_bounded(bounds)? {
                return Ok(false);
            }
        }
        if !nullable && self.is_null() {
            return Ok(false);
        }
        Ok(true)
    }
}

/// Checks if a value is null.
///
/// Since [`crate::domains::AtomDomain`] may or may not contain null values,
/// this trait is necessary for its member check.
pub trait CheckNull {
    /// # Proof Definition
    /// For any `value` of type `Self`, returns true if is null, otherwise false.
    fn is_null(&self) -> bool;
}

/// Defines an example null value inherent to `Self`.
pub trait InherentNull: CheckNull {
    /// # Proof Definition
    /// NULL is a constant such that `Self::is_null(Self::NULL)` is true.
    const NULL: Self;
}

/// ProductOrd is well-defined on types that are Ord on their non-null values.
///
/// The framework provides a way to ensure values are non-null at runtime.
/// This trait should only be used when the framework can rely on these assurances.
/// ProductOrd shares the same interface as Ord, but with a total_ prefix on methods
pub trait ProductOrd: PartialOrd + Sized {
    /// # Proof Definition
    /// For any two values `self` and `other` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if either `self` or `other` are null.
    /// Otherwise `out` is the [`Ordering`] of `self` and `other` as defined by [`PartialOrd`].
    fn total_cmp(&self, other: &Self) -> Fallible<Ordering>;

    /// # Proof Definition
    /// For any two values `self` and `other` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if either `self` or `other` are null.
    /// Otherwise returns `Some(out)` where `out` is the greater of `self` and `other` as defined by [`PartialOrd`].
    fn total_max(self, other: Self) -> Fallible<Self> {
        max_by(self, other, ProductOrd::total_cmp)
    }

    /// # Proof Definition
    /// For any two values `self` and `other` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if either `self` or `other` are null.
    /// Otherwise returns `Some(out)` where `out` is the lesser of `self` and `other` as defined by [`PartialOrd`].
    fn total_min(self, other: Self) -> Fallible<Self> {
        min_by(self, other, ProductOrd::total_cmp)
    }

    /// # Proof Definition
    /// For any three values `self`, `min` and `max` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if any of `self`, `min` or `max` are null.
    /// Otherwise returns `Some(out)` where `out` is `min` if $self \lt min$, `max` if $self \gt max$, or else `self`.
    fn total_clamp(self, min: Self, max: Self) -> Fallible<Self> {
        if min > max {
            return fallible!(FailedFunction, "min cannot be greater than max");
        }
        Ok(if let Ordering::Less = self.total_cmp(&min)? {
            min
        } else if let Ordering::Greater = self.total_cmp(&max)? {
            max
        } else {
            self
        })
    }

    /// # Proof Definition
    /// For any two values `self` and `other` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if either `self` or `other` are null.
    /// Otherwise returns `Ok(out)` where `out` is true if $self \lt other$.
    fn total_lt(&self, other: &Self) -> Fallible<bool> {
        Ok(self.total_cmp(other)?.is_lt())
    }

    /// # Proof Definition
    /// For any two values `self` and `other` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if either `self` or `other` are null.
    /// Otherwise returns `Ok(out)` where `out` is true if $self \le other$.
    fn total_le(&self, other: &Self) -> Fallible<bool> {
        Ok(self.total_cmp(other)?.is_le())
    }

    /// # Proof Definition
    /// For any two values `self` and `other` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if either `self` or `other` are null.
    /// Otherwise returns `Ok(out)` where `out` is true if $self \gt other$.
    fn total_gt(&self, other: &Self) -> Fallible<bool> {
        Ok(self.total_cmp(other)?.is_gt())
    }

    /// # Proof Definition
    /// For any two values `self` and `other` of type `Self`, returns `Ok(out)` or `Err(e)`.
    /// The implementation returns `Err(e)` if either `self` or `other` are null.
    /// Otherwise returns `Ok(out)` where `out` is true if $self \ge other$.
    fn total_ge(&self, other: &Self) -> Fallible<bool> {
        Ok(self.total_cmp(other)?.is_ge())
    }
}

/// Bitwise consts and decompositions for float types.
pub trait FloatBits: Copy + Sized + ExactIntCast<Self::Bits> {
    /// # Proof Definition
    /// An associated type that captures the bit representation of Self.
    type Bits: Copy
        + One
        + Zero
        + Eq
        + Shr<Output = Self::Bits>
        + Shl<Output = Self::Bits>
        + BitAnd<Output = Self::Bits>
        + BitOr<Output = Self::Bits>
        + Sub<Output = Self::Bits>
        + From<bool>
        + Into<IBig>
        + PartialOrd;
    /// # Proof Definition
    /// A constant equal to the number of bits in exponent.
    const EXPONENT_BITS: Self::Bits;
    /// # Proof Definition
    /// A constant equal to the number of bits in mantissa.
    ///
    /// # Note
    /// This should be equal to Self::MANTISSA_DIGITS - 1, because of the implied leading bit.
    const MANTISSA_BITS: Self::Bits;

    /// # Proof Definition
    /// A constant equal to the bias correction of the exponent.
    const EXPONENT_BIAS: Self::Bits;

    /// # Proof Definition
    /// For any `self` of type `Self`, returns true if `self` is positive, otherwise false.
    fn sign(self) -> bool {
        !(self.to_bits() & (Self::Bits::one() << (Self::EXPONENT_BITS + Self::MANTISSA_BITS)))
            .is_zero()
    }

    /// # Proof Definition
    /// For any `self` of type `Self`, returns the bits representing the exponent, without bias correction.
    fn raw_exponent(self) -> Self::Bits {
        // (shift the exponent to the rightmost bits) & (mask away everything but the trailing EXPONENT_BITS)
        (self.to_bits() >> Self::MANTISSA_BITS)
            & ((Self::Bits::one() << Self::EXPONENT_BITS) - Self::Bits::one())
    }

    /// # Proof Definition
    /// For any `self` of type `Self`, returns the bits representing the mantissa.
    fn mantissa(self) -> Self::Bits {
        // (bits) & (mask away everything but the trailing MANTISSA_BITS)
        self.to_bits() & ((Self::Bits::one() << Self::MANTISSA_BITS) - Self::Bits::one())
    }

    /// # Proof Definition
    /// For any set of arguments, returns the corresponding floating-point number according to IEEE-754.
    ///
    /// # Notes
    /// Assumes that the bits to the left of the bit range of the raw_exponent and mantissa are not set.
    fn from_raw_components(sign: bool, raw_exponent: Self::Bits, mantissa: Self::Bits) -> Self {
        // shift the sign to the leading bit
        let sign = Self::Bits::from(sign) << (Self::EXPONENT_BITS + Self::MANTISSA_BITS);
        // shift the exponent to the next EXPONENT_BITS
        let raw_exponent = raw_exponent << Self::MANTISSA_BITS;

        // mantissa is already in place, bit-or them together
        Self::from_bits(sign | raw_exponent | mantissa)
    }

    /// # Proof Definition
    /// For any `self` of type `Self`, decomposes the type into the sign, raw exponent and mantissa.
    fn to_raw_components(self) -> (bool, Self::Bits, Self::Bits) {
        (self.sign(), self.raw_exponent(), self.mantissa())
    }

    /// # Proof Definition
    /// For any `self` of type `Self`, retrieves the corresponding bit representation.
    fn to_bits(self) -> Self::Bits;

    /// # Proof Definition
    /// For any `bits` of the associated `Bits` type, returns the floating-point number corresponding to `bits`.
    fn from_bits(bits: Self::Bits) -> Self;
}

impl<T> CollectionSize for Vec<T> {
    fn size(&self) -> usize {
        self.len()
    }
}

impl<K, V> CollectionSize for HashMap<K, V> {
    fn size(&self) -> usize {
        self.len()
    }
}

macro_rules! impl_CheckAtom_number {
    ($($ty:ty)+) => ($(impl CheckAtom for $ty {
        fn is_bounded(&self, bounds: Bounds<Self>) -> Fallible<bool> {
            bounds.member(self)
        }
    })+)
}

impl_CheckAtom_number!(f32 f64 i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

#[cfg(feature = "polars")]
impl_CheckAtom_number!(chrono::NaiveDate);
#[cfg(feature = "polars")]
impl_CheckAtom_number!(chrono::NaiveTime);

impl CheckAtom for (f32, f32) {
    fn is_bounded(&self, bounds: Bounds<Self>) -> Fallible<bool> {
        bounds.member(self)
    }
}
impl CheckAtom for (f64, f64) {
    fn is_bounded(&self, bounds: Bounds<Self>) -> Fallible<bool> {
        bounds.member(self)
    }
}
impl_CheckAtom_number!(RBig IBig);

macro_rules! impl_CheckAtom_simple {
    ($($ty:ty)+) => ($(impl CheckAtom for $ty {})+)
}
impl_CheckAtom_simple!(bool String char &str);

macro_rules! impl_CheckNull_for_non_nullable {
    ($($ty:ty),+) => {
        $(impl CheckNull for $ty {
            #[inline]
            fn is_null(&self) -> bool {false}
        })+
    }
}
impl_CheckNull_for_non_nullable!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool, String, &str, char, usize, isize
);

#[cfg(feature = "polars")]
impl_CheckNull_for_non_nullable!(chrono::NaiveDate);
#[cfg(feature = "polars")]
impl_CheckNull_for_non_nullable!(chrono::NaiveTime);

impl<T1: CheckNull, T2: CheckNull> CheckNull for (T1, T2) {
    fn is_null(&self) -> bool {
        self.0.is_null() || self.1.is_null()
    }
}
impl<T: CheckNull> CheckNull for Option<T> {
    #[inline]
    fn is_null(&self) -> bool {
        if let Some(v) = self {
            v.is_null()
        } else {
            true
        }
    }
}
macro_rules! impl_CheckNull_for_float {
    ($($ty:ty),+) => {
        $(impl CheckNull for $ty {
            #[inline]
            fn is_null(&self) -> bool {self.is_nan()}
        })+
    }
}
impl_CheckNull_for_float!(f64, f32);
impl CheckNull for RBig {
    fn is_null(&self) -> bool {
        self.denominator().is_zero()
    }
}
impl CheckNull for IBig {
    fn is_null(&self) -> bool {
        false
    }
}
#[cfg(feature = "contrib")]
impl<Q, A> CheckNull for Queryable<Q, A> {
    #[inline]
    fn is_null(&self) -> bool {
        false
    }
}

// TRAIT InherentNull
macro_rules! impl_inherent_null_float {
    ($($ty:ty),+) => ($(impl InherentNull for $ty {
        const NULL: Self = Self::NAN;
    })+)
}
impl_inherent_null_float!(f64, f32);

// TRAIT ProductOrd
macro_rules! impl_ProductOrd_for_ord {
    ($($ty:ty),*) => {$(impl ProductOrd for $ty {
        fn total_cmp(&self, other: &Self) -> Fallible<Ordering> {Ok(Ord::cmp(self, other))}
    })*}
}
impl_ProductOrd_for_ord!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl_ProductOrd_for_ord!(RBig, IBig);

#[cfg(feature = "polars")]
impl_ProductOrd_for_ord!(chrono::NaiveDate);
#[cfg(feature = "polars")]
impl_ProductOrd_for_ord!(chrono::NaiveTime);

macro_rules! impl_total_ord_for_float {
    ($($ty:ty),*) => {
        $(impl ProductOrd for $ty {
            fn total_cmp(&self, other: &Self) -> Fallible<Ordering> {
                PartialOrd::partial_cmp(self, other)
                    .ok_or_else(|| err!(FailedFunction, concat!(stringify!($ty), " cannot not be null when clamping.")))
            }
        })*
    }
}
impl_total_ord_for_float!(f64, f32);

fn max_by<T, F: FnOnce(&T, &T) -> Fallible<Ordering>>(v1: T, v2: T, compare: F) -> Fallible<T> {
    compare(&v1, &v2).map(|cmp| match cmp {
        Ordering::Less | Ordering::Equal => v2,
        Ordering::Greater => v1,
    })
}

fn min_by<T, F: FnOnce(&T, &T) -> Fallible<Ordering>>(v1: T, v2: T, compare: F) -> Fallible<T> {
    compare(&v1, &v2).map(|cmp| match cmp {
        Ordering::Less | Ordering::Equal => v1,
        Ordering::Greater => v2,
    })
}

impl<T1: ProductOrd + Debug, T2: ProductOrd + Debug> ProductOrd for (T1, T2) {
    fn total_cmp(&self, other: &Self) -> Fallible<Ordering> {
        use Ordering::*;
        Ok(
            match (self.0.total_cmp(&other.0)?, self.1.total_cmp(&other.1)?) {
                (Equal, Equal) => Equal,
                (Less | Equal, Less | Equal) => Less,
                (Greater | Equal, Greater | Equal) => Greater,
                _ => {
                    return fallible!(
                        FailedFunction,
                        "unknown ordering between {:?} and {:?}",
                        self,
                        other
                    )
                }
            },
        )
    }
}

// TRAIT FloatBits
impl FloatBits for f64 {
    type Bits = u64;
    const EXPONENT_BITS: u64 = 11;
    const MANTISSA_BITS: u64 = 52;
    const EXPONENT_BIAS: u64 = 1023;

    fn to_bits(self) -> Self::Bits {
        self.to_bits()
    }
    fn from_bits(bits: Self::Bits) -> Self {
        Self::from_bits(bits)
    }
}

impl FloatBits for f32 {
    type Bits = u32;
    const EXPONENT_BITS: u32 = 8;
    const MANTISSA_BITS: u32 = 23;
    const EXPONENT_BIAS: u32 = 127;

    fn to_bits(self) -> Self::Bits {
        self.to_bits()
    }
    fn from_bits(bits: Self::Bits) -> Self {
        Self::from_bits(bits)
    }
}
