#[cfg(feature = "use-mpfr")]
use rug::ops::{AddAssignRound, DivAssignRound, MulAssignRound, SubAssignRound};

use crate::error::Fallible;
use crate::traits::CastInternalReal;

/// Computes the absolute value and returns an error if overflowing.
pub trait AlertingAbs: Sized {
    fn alerting_abs(&self) -> Fallible<Self>;
}

/// Addition that returns an error if overflowing.
pub trait AlertingAdd: Sized {
    /// Returns `Ok(self + v)` if the result does not overflow, else `Err(Error)`
    fn alerting_add(&self, v: &Self) -> Fallible<Self>;
}

/// Subtraction that returns an error if overflowing.
pub trait AlertingSub: Sized {
    /// Returns `Ok(self - v)` if the result does not overflow, else `Err(Error)`
    fn alerting_sub(&self, v: &Self) -> Fallible<Self>;
}

/// Multiplication that returns an error if overflowing.
pub trait AlertingMul: Sized {
    /// Returns `Ok(self * v)` if the result does not overflow, else `Err(Error)`
    fn alerting_mul(&self, v: &Self) -> Fallible<Self>;
}

/// Division that returns an error if overflowing.
pub trait AlertingDiv: Sized {
    /// Returns `Ok(self / v)` if the result does not overflow, else `Err(Error)`
    fn alerting_div(&self, v: &Self) -> Fallible<Self>;
}

/// Addition that saturates at the numeric bounds instead of overflowing.
pub trait SaturatingAdd: Sized {
    /// Returns `self + v`, saturating at the relevant high or low boundary of the type.
    fn saturating_add(&self, v: &Self) -> Self;
}

/// Exponentiates with specified rounding that returns an error if overflowing.
pub trait InfExp: Sized {
    fn inf_exp(self) -> Fallible<Self>;
    fn neg_inf_exp(self) -> Fallible<Self>;
}

/// Computes the logarithm with specified rounding that returns an error if overflowing.
pub trait InfLn: Sized {
    fn inf_ln(self) -> Fallible<Self>;
    fn neg_inf_ln(self) -> Fallible<Self>;
}

/// Computes the square root with specified rounding that returns an error if overflowing.
pub trait InfSqrt: Sized {
    fn inf_sqrt(self) -> Fallible<Self>;
    fn neg_inf_sqrt(self) -> Fallible<Self>;
}

/// Performs addition with rounding towards infinity that returns an error if overflowing.
pub trait InfAdd: Sized {
    /// Alerting addition with rounding towards infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn inf_add(&self, v: &Self) -> Fallible<Self>;
    /// Alerting addition with rounding towards -infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn neg_inf_add(&self, v: &Self) -> Fallible<Self>;
}

/// Performs subtraction with rounding towards infinity that returns an error if overflowing.
pub trait InfSub: Sized {
    /// Alerting subtraction with rounding towards infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn inf_sub(&self, v: &Self) -> Fallible<Self>;
    /// Alerting subtraction with rounding towards -infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn neg_inf_sub(&self, v: &Self) -> Fallible<Self>;
}

/// Performs multiplication with rounding that returns an error if overflowing.
pub trait InfMul: Sized {
    /// Alerting multiplication with rounding towards infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn inf_mul(&self, v: &Self) -> Fallible<Self>;
    /// Alerting multiplication with rounding towards -infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn neg_inf_mul(&self, v: &Self) -> Fallible<Self>;
}

/// Performs division with rounding towards infinity that returns an error if overflowing.
pub trait InfDiv: Sized {
    /// Alerting division with rounding towards infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn inf_div(&self, v: &Self) -> Fallible<Self>;
    /// Alerting division with rounding towards -infinity.
    /// Returns `Ok` if the result does not overflow, else `Err`
    fn neg_inf_div(&self, v: &Self) -> Fallible<Self>;
}

// BEGIN IMPLEMENTATIONS

// TRAIT AlertingAbs
macro_rules! impl_alerting_abs_signed_int {
    ($($ty:ty),+) => ($(impl AlertingAbs for $ty {
        fn alerting_abs(&self) -> Fallible<Self> {
            self.checked_abs().ok_or_else(|| err!(FailedFunction,
                "the corresponding positive value for {} is out of range", self))
        }
    })+)
}
impl_alerting_abs_signed_int!(i8, i16, i32, i64, i128);
macro_rules! impl_alerting_abs_unsigned_int {
    ($($ty:ty),+) => ($(impl AlertingAbs for $ty {
        fn alerting_abs(&self) -> Fallible<Self> {
            Ok(*self)
        }
    })+)
}
impl_alerting_abs_unsigned_int!(u8, u16, u32, u64, u128, usize);
macro_rules! impl_alerting_abs_float {
    ($($ty:ty),+) => ($(impl AlertingAbs for $ty {
        fn alerting_abs(&self) -> Fallible<Self> {
            Ok(self.abs())
        }
    })+)
}
impl_alerting_abs_float!(f32, f64);



// TRAIT Alerting*, Saturating*
macro_rules! impl_alerting_int {
    ($($t:ty),+) => {
        $(impl SaturatingAdd for $t {
            #[inline]
            fn saturating_add(&self, v: &Self) -> Self {
                <$t>::saturating_add(*self, *v)
            }
        })+
        $(impl AlertingMul for $t {
            #[inline]
            fn alerting_mul(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_mul(*self, *v).ok_or_else(|| err!(
                    FailedFunction,
                    "{} * {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingDiv for $t {
            #[inline]
            fn alerting_div(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_div(*self, *v).ok_or_else(|| err!(
                    FailedFunction,
                    "{} / {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingAdd for $t {
            #[inline]
            fn alerting_add(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_add(*self, *v).ok_or_else(|| err!(
                    FailedFunction,
                    "{} + {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingSub for $t {
            #[inline]
            fn alerting_sub(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_sub(*self, *v).ok_or_else(|| err!(
                    FailedFunction,
                    "{} - {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+
    };
}
impl_alerting_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize);
macro_rules! impl_alerting_float {
    ($($t:ty),+) => {
        $(impl SaturatingAdd for $t {
            fn saturating_add(&self, v: &Self) -> Self {
                (self + v).clamp(<$t>::MIN, <$t>::MAX)
            }
        })+
        $(impl AlertingMul for $t {
            fn alerting_mul(&self, v: &Self) -> Fallible<Self> {
                let y = self * v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    FailedFunction,
                    "{} * {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingDiv for $t {
            fn alerting_div(&self, v: &Self) -> Fallible<Self> {
                let y = self / v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    FailedFunction,
                    "{} / {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingAdd for $t {
            fn alerting_add(&self, v: &Self) -> Fallible<Self> {
                let y = self + v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    FailedFunction,
                    "{} + {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingSub for $t {
            fn alerting_sub(&self, v: &Self) -> Fallible<Self> {
                let y = self - v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    FailedFunction,
                    "{} - {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
    }
}
impl_alerting_float!(f32, f64);


// TRAIT InfSqrt, InfLn, InfExp (univariate)
macro_rules! impl_float_inf_uni {
    ($($ty:ty),+; $name:ident, $method_inf:ident, $method_neg_inf:ident, $op:ident, $fallback:ident) => {
        $(
        #[cfg(feature="use-mpfr")]
        impl $name for $ty {
            fn $method_inf(self) -> Fallible<Self> {
                use rug::float::Round::Up;
                let mut this = self.into_internal();
                this.$op(Up);
                let this = Self::from_internal(this);
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "() is not finite. Consider tightening your parameters."),
                    self))
            }
            fn $method_neg_inf(self) -> Fallible<Self> {
                use rug::float::Round::Down;
                let mut this = self.into_internal();
                this.$op(Down);
                let this = Self::from_internal(this);
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "() is not finite. Consider tightening your parameters."),
                    self))
            }
        }
        #[cfg(not(feature="use-mpfr"))]
        impl $name for $ty {
            fn $method_inf(self) -> Fallible<Self> {
                let this = self.$fallback();
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "() is not finite. Consider tightening your parameters."),
                    self))
            }
            fn $method_neg_inf(self) -> Fallible<Self> {
                let this = self.$fallback();
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "() is not finite. Consider tightening your parameters."),
                    self))
            }
        })+
    }
}
impl_float_inf_uni!(f64, f32; InfLn, inf_ln, neg_inf_ln, ln_round, ln);
impl_float_inf_uni!(f64, f32; InfExp, inf_exp, neg_inf_exp, exp_round, exp);
impl_float_inf_uni!(f64, f32; InfSqrt, inf_sqrt, neg_inf_sqrt, sqrt_round, sqrt);


// TRAIT InfAdd, InfSub, InfMul, InfDiv (bivariate)
macro_rules! impl_int_inf {
    ($ty:ty, $name:ident, $method_inf:ident, $method_neg_inf:ident, $func:ident) =>
        (impl $name for $ty {
            fn $method_inf(&self, other: &Self) -> Fallible<Self> {
                self.$func(other)
            }
            fn $method_neg_inf(&self, other: &Self) -> Fallible<Self> {
                self.$func(other)
            }
        });
    ($($ty:ty),+) => {
        $(impl_int_inf!{$ty, InfAdd, inf_add, neg_inf_add, alerting_add})+
        $(impl_int_inf!{$ty, InfSub, inf_sub, neg_inf_sub, alerting_sub})+
        $(impl_int_inf!{$ty, InfMul, inf_mul, neg_inf_mul, alerting_mul})+
        $(impl InfDiv for $ty {
            fn inf_div(&self, other: &Self) -> Fallible<Self> {
                self.alerting_add(&1)?.alerting_div(other)
            }
            fn neg_inf_div(&self, other: &Self) -> Fallible<Self> {
                self.alerting_div(other)
            }
        })+
    }
}
impl_int_inf!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize);

macro_rules! impl_float_inf_bi {
    ($($ty:ty),+; $name:ident, $method_inf:ident, $method_neg_inf:ident, $op:ident, $fallback:ident) => {
        $(
        #[cfg(feature="use-mpfr")]
        impl $name for $ty {
            fn $method_inf(&self, other: &Self) -> Fallible<Self> {
                use rug::float::Round::Up;
                let mut this = self.into_internal();
                this.$op(other, Up);
                let this = Self::from_internal(this);
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
            fn $method_neg_inf(&self, other: &Self) -> Fallible<Self> {
                use rug::float::Round::Down;
                let mut this = self.into_internal();
                this.$op(other, Down);
                let this = Self::from_internal(this);
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
        }
        #[cfg(not(feature="use-mpfr"))]
        impl $name for $ty {
            fn $method_inf(&self, other: &Self) -> Fallible<Self> {
                let this = self.$fallback(other);
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
            fn $method_neg_inf(&self, other: &Self) -> Fallible<Self> {
                let this = self.$fallback(other);
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
        })+
    }
}
impl_float_inf_bi!(f64, f32; InfAdd, inf_add, neg_inf_add, add_assign_round, alerting_add);
impl_float_inf_bi!(f64, f32; InfSub, inf_sub, neg_inf_sub, sub_assign_round, alerting_sub);
impl_float_inf_bi!(f64, f32; InfMul, inf_mul, neg_inf_mul, mul_assign_round, alerting_mul);
impl_float_inf_bi!(f64, f32; InfDiv, inf_div, neg_inf_div, div_assign_round, alerting_div);


impl<T1: InfSub, T2: InfSub> InfSub for (T1, T2) {
    fn inf_sub(&self, v: &Self) -> Fallible<Self> {
        Ok((self.0.inf_sub(&v.0)?, self.1.inf_sub(&v.1)?))
    }

    fn neg_inf_sub(&self, v: &Self) -> Fallible<Self> {
        Ok((self.0.neg_inf_sub(&v.0)?, self.1.neg_inf_sub(&v.1)?))
    }
}