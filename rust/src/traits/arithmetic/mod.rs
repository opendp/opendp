#[cfg(feature = "use-mpfr")]
use rug::ops::{AddAssignRound, DivAssignRound, MulAssignRound, SubAssignRound, PowAssignRound};
use crate::traits::{ExactIntCast, InfCast};

use crate::error::Fallible;
#[cfg(feature="use-mpfr")]
use rug::Float;



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
    /// `self.inf_log2()` either returns `Ok(out)`, 
    /// where $out \ge \sqrt{self}$, or `Err(e)`.
    fn inf_sqrt(self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `self` of type `Self`, 
    /// `self.neg_inf_log2()` either returns `Ok(out)`, 
    /// where $out \le \sqrt{self}$, or `Err(e)`.
    fn neg_inf_sqrt(self) -> Fallible<Self>;
}

/// Fallibly raise self to the power with specified rounding.
/// 
/// Throws an error if the ideal output is not finite or representable.
pub trait InfPow: Sized + AlertingPow {
    /// # Proof Definition
    /// For any two values `self` and `p` of type `Self`, 
    /// `self.inf_pow(p)` either returns `Ok(out)`, 
    /// where $out \ge self^{p}$, or `Err(e)`.
    fn inf_pow(&self, p: &Self) -> Fallible<Self>;
    /// # Proof Definition
    /// For any two values `self` and `p` of type `Self`, 
    /// `self.neg_inf_pow(p)` either returns `Ok(out)`, 
    /// where $out \le self^{p}$, or `Err(e)`.
    fn neg_inf_pow(&self, p: &Self) -> Fallible<Self>;
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

 /// Fallible logarithm of the (argument plus one) with specified rounding.
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
            self.checked_abs().ok_or_else(|| err!(FailedFunction,
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

        $(impl AlertingPow for $t {
            #[inline]
            fn alerting_pow(&self, p: &Self) -> Fallible<Self> {
                let p = u32::exact_int_cast(*p)?;
                <$t>::checked_pow(*self, p).ok_or_else(|| err!(
                    FailedFunction,
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
        $(impl AlertingPow for $t {
            fn alerting_pow(&self, v: &Self) -> Fallible<Self> {
                let y = self.powf(*v);
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
                let mut this = Float::inf_cast(self)?;
                this.$op(Up);
                let this = Self::inf_cast(this)?;
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_inf), "() is not finite. Consider tightening your parameters."),
                    self))
            }
            fn $method_neg_inf(self) -> Fallible<Self> {
                use rug::float::Round::Down;
                let mut this = Float::neg_inf_cast(self)?;
                this.$op(Down);
                let this = Self::neg_inf_cast(this)?;
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_neg_inf), "() is not finite. Consider tightening your parameters."),
                    self))
            }
        }
        #[cfg(not(feature="use-mpfr"))]
        impl $name for $ty {
            fn $method_inf(self) -> Fallible<Self> {
                let this = self.$fallback();
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_inf), "() is not finite. Consider tightening your parameters."),
                    self))
            }
            fn $method_neg_inf(self) -> Fallible<Self> {
                let this = self.$fallback();
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_neg_inf), "() is not finite. Consider tightening your parameters."),
                    self))
            }
        })+
    }
}
impl_float_inf_uni!(f64, f32; InfLn, inf_ln, neg_inf_ln, ln_round, ln);
impl_float_inf_uni!(f64, f32; InfLog2, inf_log2, neg_inf_log2, log2_round, log2);
impl_float_inf_uni!(f64, f32; InfExp, inf_exp, neg_inf_exp, exp_round, exp);
impl_float_inf_uni!(f64, f32; InfLn1P, inf_ln_1p, neg_inf_ln_1p, ln_1p_round, ln_1p);
impl_float_inf_uni!(f64, f32; InfExpM1, inf_exp_m1, neg_inf_exp_m1, exp_m1_round, exp_m1);
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
                if other == &0 {
                    return fallible!(FailedFunction, "attempt to divide by zero");
                }
                Ok(num::Integer::div_ceil(self, other))
            }
            fn neg_inf_div(&self, other: &Self) -> Fallible<Self> {
                if other == &0 {
                    return fallible!(FailedFunction, "attempt to divide by zero");
                }
                Ok(num::Integer::div_floor(self, other))
            }
        })+
    }
}
impl_int_inf!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! impl_float_inf_bi {
    ($($ty:ty),+; $name:ident, $method_inf:ident, $method_neg_inf:ident, $op:ident, $fallback:ident) => {
        $(
        #[cfg(feature="use-mpfr")]
        impl $name for $ty {
            fn $method_inf(&self, other: &Self) -> Fallible<Self> {
                use rug::float::Round::Up;
                let mut this = Float::inf_cast(*self)?;
                this.$op(other, Up);
                let this = Self::inf_cast(this)?;
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
            fn $method_neg_inf(&self, other: &Self) -> Fallible<Self> {
                use rug::float::Round::Down;
                let mut this = Float::neg_inf_cast(*self)?;
                this.$op(other, Down);
                let this = Self::neg_inf_cast(this)?;
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_neg_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
        }
        #[cfg(not(feature="use-mpfr"))]
        impl $name for $ty {
            fn $method_inf(&self, other: &Self) -> Fallible<Self> {
                let this = self.$fallback(other)?;
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
            fn $method_neg_inf(&self, other: &Self) -> Fallible<Self> {
                let this = self.$fallback(other)?;
                this.is_finite().then(|| this).ok_or_else(|| err!(
                    FailedFunction,
                    concat!("({}).", stringify!($method_neg_inf), "({}) is not finite. Consider tightening your parameters."),
                    self, other))
            }
        })+
    }
}
impl_float_inf_bi!(f64, f32; InfAdd, inf_add, neg_inf_add, add_assign_round, alerting_add);
impl_float_inf_bi!(f64, f32; InfSub, inf_sub, neg_inf_sub, sub_assign_round, alerting_sub);
impl_float_inf_bi!(f64, f32; InfMul, inf_mul, neg_inf_mul, mul_assign_round, alerting_mul);
impl_float_inf_bi!(f64, f32; InfDiv, inf_div, neg_inf_div, div_assign_round, alerting_div);
impl_float_inf_bi!(f64, f32; InfPow, inf_pow, neg_inf_pow, pow_assign_round, alerting_pow);
