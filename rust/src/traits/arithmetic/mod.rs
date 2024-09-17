use std::ops::{Add, Div, Mul, Sub};

use crate::{
    error::Fallible,
    traits::{ExactIntCast, InfCast},
};
use dashu::{
    base::{EstimatedLog2, SquareRoot},
    float::{
        round::mode::{Down, Up},
        FBig,
    },
    integer::IBig,
};
use std::panic;

// for context on why this is used, see conversation on https://github.com/cmpute/dashu/issues/29
fn catch_unwind_silent<R>(f: impl FnOnce() -> R + panic::UnwindSafe) -> std::thread::Result<R> {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(prev_hook);
    result
}

/// Fallible absolute value that returns an error if overflowing.
///
/// This can return an error when a signed integer is the smallest negative value.
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::AlertingAbs;
/// assert!(i8::MIN.alerting_abs().is_err());
/// ```
pub trait AlertingAbs: Sized {
    /// # Proof Definition
    /// For any `self` of type `Self`, returns `Ok(out)` where $out = |self|$ or `Err(e)`.
    fn alerting_abs(&self) -> Fallible<Self>;
}

/// Fallible addition that returns an error if overflowing.
///
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::AlertingAdd;
/// assert!(i8::MAX.alerting_add(&1).is_err());
/// ```
pub trait AlertingAdd: Sized {
    /// # Proof Definition
    /// For any `self` and `v` of type `Self`,
    /// returns `Ok(self + v)` if the result does not overflow, else `Err(e)`
    fn alerting_add(&self, v: &Self) -> Fallible<Self>;
}

/// Fallible subtraction that returns an error if overflowing.
///
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::AlertingSub;
/// assert!(i8::MIN.alerting_sub(&1).is_err());
/// ```
pub trait AlertingSub: Sized {
    /// # Proof Definition
    /// For any `self` and `v` of type `Self`,
    /// returns `Ok(self - v)` if the result does not overflow, else `Err(e)`
    fn alerting_sub(&self, v: &Self) -> Fallible<Self>;
}

/// Fallible multiplication that returns an error if overflowing.
///
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::AlertingMul;
/// assert!(i8::MAX.alerting_mul(&2).is_err());
/// ```
pub trait AlertingMul: Sized {
    /// # Proof Definition
    /// For any `self` and `v` of type `Self`,
    /// returns `Ok(self * v)` if the result does not overflow, else `Err(e)`
    fn alerting_mul(&self, v: &Self) -> Fallible<Self>;
}

/// Fallible division that returns an error if overflowing.
///
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::AlertingDiv;
/// assert!(1u8.alerting_div(&0).is_err());
/// ```
pub trait AlertingDiv: Sized {
    /// # Proof Definition
    /// For any `self` and `v` of type `Self`,
    /// returns `Ok(self / v)` if the result does not overflow, else `Err(e)`
    fn alerting_div(&self, v: &Self) -> Fallible<Self>;
}

/// Fallibly raise to the power.
///
/// Returns an error if overflowing.
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::AlertingPow;
/// assert!(2u8.alerting_pow(&8).is_err());
/// ```
pub trait AlertingPow: Sized {
    /// # Proof Definition
    /// For any `self` and `v` of type `Self`,
    /// returns `Ok(self^v)` if the result does not overflow, else `Err(e)`
    fn alerting_pow(&self, p: &Self) -> Fallible<Self>;
}

/// Addition that saturates at the numeric bounds instead of overflowing.
///
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::SaturatingAdd;
/// assert_eq!(i8::MAX.saturating_add(i8::MAX), i8::MAX);
/// ```
pub trait SaturatingAdd: Sized {
    /// # Proof Definition
    /// For any `self` and `v` of type `Self`,
    /// returns `self + v`, saturating at the relevant high or low boundary of the type.
    fn saturating_add(&self, v: &Self) -> Self;
}

/// Multiplication that saturates at the numeric bounds instead of overflowing.
///
/// Avoids unrecoverable panics that could leak private information.
/// ```
/// use opendp::traits::SaturatingMul;
/// assert_eq!(i8::MAX.saturating_mul(2), i8::MAX);
/// ```
pub trait SaturatingMul: Sized {
    /// # Proof Definition
    /// For any `self` and `v` of type `Self`,
    /// returns `self * v`, saturating at the relevant high or low boundary of the type.
    fn saturating_mul(&self, v: &Self) -> Self;
}

/// Fallible exponentiation with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfExp: Sized {
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.inf_exp()` either returns `Ok(out)`,
    /// where $out \ge \exp(self)$, or `Err(e)`.
    fn inf_exp(self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.neg_inf_exp()` either returns `Ok(out)`,
    /// where $out \le \exp(self)$, or `Err(e)`.
    fn neg_inf_exp(self) -> Fallible<Self>;
}

/// Fallible natural logarithm with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfLn: Sized {
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.inf_ln()` either returns `Ok(out)`,
    /// where $out \ge \ln(self)$, or `Err(e)`.
    fn inf_ln(self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.neg_inf_ln()` either returns `Ok(out)`,
    /// where $out \le \ln(self)$, or `Err(e)`.
    fn neg_inf_ln(self) -> Fallible<Self>;
}

/// Fallible base-2 logarithm with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfLog2: Sized {
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.inf_log2()` either returns `Ok(out)`,
    /// where $out \ge \log_2(self)$, or `Err(e)`.
    fn inf_log2(self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.neg_inf_log2()` either returns `Ok(out)`,
    /// where $out \le \log_2(self)$, or `Err(e)`.
    fn neg_inf_log2(self) -> Fallible<Self>;
}

/// Fallible square root with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfSqrt: Sized {
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.inf_sqrt()` either returns `Ok(out)`,
    /// where $out \ge \sqrt{self}$, or `Err(e)`.
    fn inf_sqrt(self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.neg_inf_sqrt()` either returns `Ok(out)`,
    /// where $out \le \sqrt{self}$, or `Err(e)`.
    fn neg_inf_sqrt(self) -> Fallible<Self>;
}

/// Fallibly raise self to the power with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfPowI: Sized + AlertingPow {
    /// # Proof Definition
    /// For any two values `self` and `p` of type `Self`,
    /// `self.inf_powi(p)` either returns `Ok(out)`,
    /// where $out \ge self^{p}$, or `Err(e)`.
    fn inf_powi(&self, p: IBig) -> Fallible<Self>;
    /// # Proof Definition
    /// For any two values `self` and `p` of type `Self`,
    /// `self.neg_inf_powi(p)` either returns `Ok(out)`,
    /// where $out \le self^{p}$, or `Err(e)`.
    fn neg_inf_powi(&self, p: IBig) -> Fallible<Self>;
}

/// Fallible addition with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfAdd: Sized + AlertingAdd {
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.inf_add(v)` either returns `Ok(out)`,
    /// where $out \ge self + v$, or `Err(e)`.
    fn inf_add(&self, v: &Self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.neg_inf_add(v)` either returns `Ok(out)`,
    /// where $out \le self + v$, or `Err(e)`.
    fn neg_inf_add(&self, v: &Self) -> Fallible<Self>;
}

/// Fallible subtraction with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfSub: Sized + AlertingSub {
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.inf_sub(v)` either returns `Ok(out)`,
    /// where $out \ge self - v$, or `Err(e)`.
    fn inf_sub(&self, v: &Self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.neg_inf_sub(v)` either returns `Ok(out)`,
    /// where $out \le self - v$, or `Err(e)`.
    fn neg_inf_sub(&self, v: &Self) -> Fallible<Self>;
}

/// Fallible multiplication with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfMul: Sized + AlertingMul {
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.inf_mul(v)` either returns `Ok(out)`,
    /// where $out \ge self \cdot v$, or `Err(e)`.
    fn inf_mul(&self, v: &Self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.neg_inf_mul(v)` either returns `Ok(out)`,
    /// where $out \le self \cdot v$, or `Err(e)`.
    fn neg_inf_mul(&self, v: &Self) -> Fallible<Self>;
}

/// Fallible division with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
pub trait InfDiv: Sized + AlertingDiv {
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.inf_div(v)` either returns `Ok(out)`,
    /// where $out \ge self / v$, or `Err(e)`.
    fn inf_div(&self, v: &Self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any two values `self` and `v` of type `Self`,
    /// `self.neg_inf_div(v)` either returns `Ok(out)`,
    /// where $out \le self / v$, or `Err(e)`.
    fn neg_inf_div(&self, v: &Self) -> Fallible<Self>;
}

/// Fallibly exponentiate and subtract one with specified rounding.
///
/// Throws an error if the ideal output is not finite or representable.
/// This provides more numerical stability than computing the quantity outright.
pub trait InfExpM1: Sized {
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.inf_exp_m1()` either returns `Ok(out)`,
    /// where $out \ge \exp(self) - 1$, or `Err(e)`.
    fn inf_exp_m1(self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.neg_inf_exp_m1()` either returns `Ok(out)`,
    /// where $out \le \exp(self) - 1$, or `Err(e)`.
    fn neg_inf_exp_m1(self) -> Fallible<Self>;
}

/// Fallible logarithm of the argument plus one with specified rounding.
pub trait InfLn1P: Sized {
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.inf_ln_1p()` either returns `Ok(out)`,
    /// where $out \ge \ln(self + 1)$, or `Err(e)`.
    fn inf_ln_1p(self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `self` of type `Self`,
    /// `self.neg_inf_ln_1p()` either returns `Ok(out)`,
    /// where $out \le \ln(self + 1)$, or `Err(e)`.
    fn neg_inf_ln_1p(self) -> Fallible<Self>;
}

// BEGIN IMPLEMENTATIONS

// TRAIT AlertingAbs
macro_rules! impl_alerting_abs_signed_int {
    ($($ty:ty),+) => ($(impl AlertingAbs for $ty {
        fn alerting_abs(&self) -> Fallible<Self> {
            self.checked_abs().ok_or_else(|| err!(Overflow,
                "the corresponding positive value for {} is out of range", self))
        }
    })+)
}
impl_alerting_abs_signed_int!(i8, i16, i32, i64, i128, isize);
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
        })
        +$(impl SaturatingMul for $t {
            #[inline]
            fn saturating_mul(&self, v: &Self) -> Self {
                <$t>::saturating_mul(*self, *v)
            }
        })+
        $(impl AlertingMul for $t {
            #[inline]
            fn alerting_mul(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_mul(*self, *v).ok_or_else(|| err!(
                    Overflow,
                    "{} * {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingDiv for $t {
            #[inline]
            fn alerting_div(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_div(*self, *v).ok_or_else(|| err!(
                    Overflow,
                    "{} / {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingAdd for $t {
            #[inline]
            fn alerting_add(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_add(*self, *v).ok_or_else(|| err!(
                    Overflow,
                    "{} + {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingSub for $t {
            #[inline]
            fn alerting_sub(&self, v: &Self) -> Fallible<Self> {
                <$t>::checked_sub(*self, *v).ok_or_else(|| err!(
                    Overflow,
                    "{} - {} overflows. Consider tightening your parameters.",
                    self, v))
            }
        })+

        $(impl AlertingPow for $t {
            #[inline]
            fn alerting_pow(&self, p: &Self) -> Fallible<Self> {
                let p = u32::exact_int_cast(*p)?;
                <$t>::checked_pow(*self, p).ok_or_else(|| err!(
                    Overflow,
                    "{}.pow({}) overflows. Consider tightening your parameters.",
                    self, p))
            }
        })+
    };
}
impl_alerting_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
macro_rules! impl_alerting_float {
    ($($t:ty),+) => {
        $(impl SaturatingAdd for $t {
            fn saturating_add(&self, v: &Self) -> Self {
                (self + v).clamp(<$t>::MIN, <$t>::MAX)
            }
        })+
        $(impl SaturatingMul for $t {
            fn saturating_mul(&self, v: &Self) -> Self {
                (self * v).clamp(<$t>::MIN, <$t>::MAX)
            }
        })+
        $(impl AlertingMul for $t {
            fn alerting_mul(&self, v: &Self) -> Fallible<Self> {
                let y = self * v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    Overflow,
                    "{} * {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingDiv for $t {
            fn alerting_div(&self, v: &Self) -> Fallible<Self> {
                let y = self / v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    Overflow,
                    "{} / {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingAdd for $t {
            fn alerting_add(&self, v: &Self) -> Fallible<Self> {
                let y = self + v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    Overflow,
                    "{} + {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingSub for $t {
            fn alerting_sub(&self, v: &Self) -> Fallible<Self> {
                let y = self - v;
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    Overflow,
                    "{} - {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
        $(impl AlertingPow for $t {
            fn alerting_pow(&self, v: &Self) -> Fallible<Self> {
                let y = self.powf(*v);
                y.is_finite().then(|| y).ok_or_else(|| err!(
                    Overflow,
                    "{} - {} is not finite. Consider tightening your parameters.",
                    self, v))
            }
        })+
    }
}
impl_alerting_float!(f32, f64);

trait Log2 {
    fn log2(self) -> Self;
}

impl Log2 for FBig<Down> {
    fn log2(self) -> Self {
        Self::try_from(self.log2_bounds().0).unwrap()
        // If you implement via log rules, the bound is looser than dashu's EstimatedLog2.
        //    However, dashu's EstimatedLog2 matches MPFR.
        //    using log_b(x) = ln(x) / ln(b):
        // self.ln() / FBig::<Up>::from(2).ln().with_rounding::<Down>()
    }
}
impl Log2 for FBig<Up> {
    fn log2(self) -> Self {
        Self::try_from(self.log2_bounds().1).unwrap()
    }
}

// TRAIT InfSqrt, InfLn, InfExp (univariate)
macro_rules! impl_float_inf_uni {
    ($($ty:ty),+; $name:ident, $method_inf:ident, $method_neg_inf:ident, $op:ident) => {
        $(
        impl $name for $ty {
            fn $method_inf(self) -> Fallible<Self> {
                let not_finite = || err!(
                    Overflow,
                    concat!("({}).", stringify!($method_inf), "() is not finite. Consider tightening your parameters."),
                    self);
                if !self.$op().is_finite() {
                    return Err(not_finite());
                }
                let lhs = FBig::<Up>::inf_cast(self)?.with_precision(<$ty>::MANTISSA_DIGITS as usize).value();
                let Ok(output) = catch_unwind_silent(|| lhs.$op()) else {
                    return Err(not_finite())
                };
                let output = Self::inf_cast(output)?;
                output.is_finite().then(|| output).ok_or_else(not_finite)
            }
            fn $method_neg_inf(self) -> Fallible<Self> {
                let not_finite = || err!(
                    Overflow,
                    concat!("({}).", stringify!($method_neg_inf), "() is not finite. Consider tightening your parameters."),
                    self);
                if !self.$op().is_finite() {
                    return Err(not_finite());
                }
                let lhs = FBig::<Down>::inf_cast(self)?;
                let Ok(output) = catch_unwind_silent(|| lhs.$op()) else {
                    return Err(not_finite())
                };
                let output = Self::neg_inf_cast(output)?;
                output.is_finite().then(|| output).ok_or_else(not_finite)
            }
        })+
    }
}
impl_float_inf_uni!(f64, f32; InfLn, inf_ln, neg_inf_ln, ln);
impl_float_inf_uni!(f64, f32; InfLog2, inf_log2, neg_inf_log2, log2);
impl_float_inf_uni!(f64, f32; InfLn1P, inf_ln_1p, neg_inf_ln_1p, ln_1p);
impl_float_inf_uni!(f64, f32; InfExpM1, inf_exp_m1, neg_inf_exp_m1, exp_m1);
impl_float_inf_uni!(f64, f32; InfSqrt, inf_sqrt, neg_inf_sqrt, sqrt);

// these implementations are expanded to catch errors in underflow.
// when the input is very negative, resulting in an underflow, the output is min subnormal
impl InfExp for f64 {
    fn inf_exp(self) -> Fallible<Self> {
        let not_finite = || {
            err!(
                Overflow,
                "({}).inf_exp() is not finite. Consider tightening your parameters.",
                self
            )
        };
        if !self.exp().is_finite() {
            return Err(not_finite());
        }
        let lhs = FBig::<Up>::inf_cast(self)?
            .with_precision(<f64>::MANTISSA_DIGITS as usize)
            .value();
        let Ok(output) = catch_unwind_silent(|| lhs.exp()) else {
            if self.is_sign_negative() {
                return Ok(f64::from_bits(1));
            }
            return Err(not_finite());
        };
        let output = Self::inf_cast(output)?;
        output.is_finite().then(|| output).ok_or_else(not_finite)
    }

    fn neg_inf_exp(self) -> Fallible<Self> {
        let not_finite = || {
            err!(
                Overflow,
                "({}).neg_inf_exp() is not finite. Consider tightening your parameters.",
                self
            )
        };
        if !self.exp().is_finite() {
            return Err(not_finite());
        }
        let lhs = FBig::<Down>::inf_cast(self)?;
        let Ok(output) = catch_unwind_silent(|| lhs.exp()) else {
            if self.is_sign_negative() {
                return Ok(0.0);
            }
            return Err(not_finite());
        };
        let output = Self::neg_inf_cast(output)?;
        output.is_finite().then(|| output).ok_or_else(not_finite)
    }
}

impl InfExp for f32 {
    fn inf_exp(self) -> Fallible<Self> {
        let not_finite = || {
            err!(
                Overflow,
                "({}).inf_exp() is not finite. Consider tightening your parameters.",
                self
            )
        };
        if !self.exp().is_finite() {
            return Err(not_finite());
        }
        let lhs = FBig::<Up>::inf_cast(self)?
            .with_precision(<f32>::MANTISSA_DIGITS as usize)
            .value();
        let Ok(output) = catch_unwind_silent(|| lhs.exp()) else {
            if self.is_sign_negative() {
                return Ok(f32::from_bits(1));
            }
            return Err(not_finite());
        };
        let output = Self::inf_cast(output)?;
        output.is_finite().then(|| output).ok_or_else(not_finite)
    }
    fn neg_inf_exp(self) -> Fallible<Self> {
        let not_finite = || {
            err!(
                Overflow,
                "({}).neg_inf_exp() is not finite. Consider tightening your parameters.",
                self
            )
        };
        if !self.exp().is_finite() {
            return Err(not_finite());
        }
        let lhs = FBig::<Down>::inf_cast(self)?;
        let Ok(output) = catch_unwind_silent(|| lhs.exp()) else {
            if self.is_sign_negative() {
                return Ok(0.0);
            }
            return Err(not_finite());
        };
        let output = Self::neg_inf_cast(output)?;
        output.is_finite().then(|| output).ok_or_else(not_finite)
    }
}

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
                if other == &0 {
                    return fallible!(Overflow, "attempt to divide by zero");
                }
                Ok(num::Integer::div_ceil(self, other))
            }
            fn neg_inf_div(&self, other: &Self) -> Fallible<Self> {
                if other == &0 {
                    return fallible!(Overflow, "attempt to divide by zero");
                }
                Ok(num::Integer::div_floor(self, other))
            }
        })+
    }
}
impl_int_inf!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! impl_float_inf_bi {
    ($($ty:ty),+; $name:ident, $method_inf:ident, $method_neg_inf:ident, $op:ident) => {
        $(impl $name for $ty {
            fn $method_inf(&self, other: &Self) -> Fallible<Self> {
                let not_finite = || err!(
                    Overflow,
                    concat!("({}).", stringify!($method_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other);
                if !self.$op(other).is_finite() {
                    return Err(not_finite());
                }
                let lhs = FBig::<Up>::try_from(*self)?;
                let rhs = FBig::<Up>::try_from(*other)?;
                let Ok(output) = catch_unwind_silent(|| lhs.$op(rhs)) else {
                    return Err(not_finite())
                };
                let output = Self::inf_cast(output)?;
                output.is_finite().then(|| output).ok_or_else(not_finite)
            }
            fn $method_neg_inf(&self, other: &Self) -> Fallible<Self> {
                let not_finite = || err!(
                    Overflow,
                    concat!("({}).", stringify!($method_neg_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other);
                if !self.$op(other).is_finite() {
                    return Err(not_finite());
                }
                let lhs = FBig::<Down>::try_from(*self)?;
                let rhs = FBig::<Down>::try_from(*other)?;
                let Ok(output) = catch_unwind_silent(|| lhs.$op(rhs)) else {
                    return Err(not_finite())
                };
                let output = Self::neg_inf_cast(output)?;
                output.is_finite().then(|| output).ok_or_else(not_finite)
            }
        })+
    }
}
impl_float_inf_bi!(f64, f32; InfAdd, inf_add, neg_inf_add, add);
impl_float_inf_bi!(f64, f32; InfSub, inf_sub, neg_inf_sub, sub);
impl_float_inf_bi!(f64, f32; InfMul, inf_mul, neg_inf_mul, mul);
impl_float_inf_bi!(f64, f32; InfDiv, inf_div, neg_inf_div, div);

macro_rules! impl_float_inf_bi_ibig {
    ($($ty:ty),+; $name:ident, $method_inf:ident, $method_neg_inf:ident, $op:ident) => {
        $(impl $name for $ty {
            fn $method_inf(&self, other: IBig) -> Fallible<Self> {
                let not_finite = || err!(
                    Overflow,
                    concat!("({}).", stringify!($method_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other);
                if !self.is_finite() {
                    return Err(not_finite());
                }
                let lhs = FBig::<Up>::try_from(*self)?;
                let Ok(output) = catch_unwind_silent(|| lhs.$op(other.clone())) else {
                    return Err(not_finite())
                };
                let output = Self::inf_cast(output)?;
                output.is_finite().then(|| output).ok_or_else(not_finite)
            }
            fn $method_neg_inf(&self, other: IBig) -> Fallible<Self> {
                let not_finite = || err!(
                    Overflow,
                    concat!("({}).", stringify!($method_neg_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other);
                if !self.is_finite() {
                    return Err(not_finite());
                }
                let lhs = FBig::<Down>::try_from(*self)?;
                let Ok(output) = catch_unwind_silent(|| lhs.$op(other.clone())) else {
                    return Err(not_finite())
                };
                let output = Self::neg_inf_cast(output)?;
                output.is_finite().then(|| output).ok_or_else(not_finite)
            }
        })+
    }
}
impl_float_inf_bi_ibig!(f64, f32; InfPowI, inf_powi, neg_inf_powi, powi);
