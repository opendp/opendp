use std::convert::TryFrom;
use std::ops::{Div, Mul, Sub, Shr, Shl, BitAnd};

use num::{NumCast, One, Zero};

use crate::error::Fallible;
use std::cmp::{Ordering};

/// A type that can be used as a stability or privacy constant to scale a distance.
/// Encapsulates the necessary traits for the new_from_constant method on relations.
/// Making a relation from a constant has the general form
///     d_out >= QO::distance_cast(d_in) * c    (where d_out and c have type QO: DistanceConstant)
/// Computing this needs all of the traits DistanceConstant inherits from:
/// - InfCast<QI>: casting where the distance after the cast is gte the distance before the cast
/// - QO also clearly needs to support Mul and PartialOrd used in the general form above.
/// - Div is used for the backward map:
///     How do you translate d_out to a d_in that can be used as a hint? |d_out| d_out / c
pub trait DistanceConstant<TI>: 'static + Clone + InfCast<TI> + Div<Output=Self> + Mul<Output=Self> + TotalOrd
    where TI: InfCast<Self> {}
impl<TI: InfCast<Self>, TO: 'static + Clone + InfCast<TI> + Div<Output=Self> + Mul<Output=Self> + TotalOrd> DistanceConstant<TI> for TO {}

// TODO: Maybe this should be renamed to something more specific to budgeting, and add negative checks? -Mike
pub trait FallibleSub<Rhs = Self> {
    type Output;
    fn fallible_sub(self, rhs: Rhs) -> Fallible<Self::Output>;
}

macro_rules! impl_fallible_sub {
    ($($ty:ty),+) => ($(
        impl<Rhs> FallibleSub<Rhs> for $ty where $ty: Sub<Rhs, Output=$ty> {
            type Output = $ty;
            fn fallible_sub(self, rhs: Rhs) -> Fallible<Self::Output> {
                Ok(self - rhs)
            }
        }
        // impl<'a, Rhs> FallibleSub<&'a Rhs> for &'a $ty where &'a $ty: Sub<&'a Rhs, Output=$ty> {
        //     type Output = $ty;
        //     fn sub(self, rhs: &'a Rhs) -> Fallible<Self::Output> {
        //         Ok(self - rhs)
        //     }
        // }
    )+)
}
impl_fallible_sub!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl<T0, T1, Rhs0, Rhs1> FallibleSub<(Rhs0, Rhs1)> for (T0, T1) where T0: Sub<Rhs0>, T1: Sub<Rhs1> {
    type Output = (T0::Output, T1::Output);
    fn fallible_sub(self, rhs: (Rhs0, Rhs1)) -> Fallible<Self::Output> {
        Ok((self.0 - rhs.0, self.1 - rhs.1))
    }
}

impl<'a, T0, T1, Rhs0, Rhs1> FallibleSub<&'a (Rhs0, Rhs1)> for (T0, T1) where T0: Sub<&'a Rhs0>, T1: Sub<&'a Rhs1> {
    type Output = (T0::Output, T1::Output);
    fn fallible_sub(self, rhs: &'a (Rhs0, Rhs1)) -> Fallible<Self::Output> {
        Ok((self.0 - &rhs.0, self.1 - &rhs.1))
    }
}

/// A type that can be used as a measure distance.
pub trait MeasureDistance: PartialOrd + for<'a> FallibleSub<&'a Self, Output=Self> {}
impl<T> MeasureDistance for T where T: PartialOrd + for<'a> FallibleSub<&'a Self, Output=Self> {}

/// A type that can be used as a metric distance.
pub trait MetricDistance: PartialOrd {}
impl<T> MetricDistance for T where T: PartialOrd {}


pub trait Abs { fn abs(self) -> Self; }
macro_rules! impl_abs_method {
    ($($ty:ty),+) => ($(impl Abs for $ty { fn abs(self) -> Self {self.abs()} })+)
}
impl_abs_method!(f64, f32);

macro_rules! impl_abs_self {
    ($($ty:ty),+) => ($(impl Abs for $ty { fn abs(self) -> Self {self} })+)
}
impl_abs_self!(u8, u16, u32, u64, u128);

macro_rules! impl_abs_int {
    ($($ty:ty),+) => ($(impl Abs for $ty {
        fn abs(self) -> Self {
            if self == Self::MIN {
                Self::MAX
            } else {
                self.abs()
            }
        }
    })+)
}
impl_abs_int!(i8, i16, i32, i64, i128);

// https://docs.google.com/spreadsheets/d/1DJohiOI3EVHjwj8g4IEdFZVf7MMyFk_4oaSyjTfkO_0/edit?usp=sharing
macro_rules! cartesian {
    // base case
    (@[$(($a1:tt, $a2:tt))*] [] $b:tt $init_b:tt $submacro:tt) =>
        ($($submacro!{$a1, $a2})*);
    // when b empty, strip off an "a" and refill b from init_b
    (@$out:tt [$a:tt, $($at:tt)*] [] $init_b:tt $submacro:tt) =>
        (cartesian!{@$out [$($at)*] $init_b $init_b $submacro});
    // strip off a "b" and add a pair to $out that consists of the first "a" and first "b"
    (@[$($out:tt)*] [$a:tt, $($at:tt)*] [$b:tt, $($bt:tt)*] $init_b:tt $submacro:tt) =>
        (cartesian!{@[$($out)* ($a, $b)] [$a, $($at)*] [$($bt)*] $init_b $submacro});

    // recurse down diagonal
    (@diag[$($start_a:tt),*], [$mid_a:tt, $($end_a:tt),*], [$($start_b:tt),*], [$mid_b:tt, $($end_b:tt),*], $lower:tt, $diag:tt, $upper:tt) => {
        $($lower!($mid_a, $start_b);)*
        $diag!($mid_a, $mid_b);
        $($upper!($mid_a, $end_b);)*
        cartesian!{@diag[$($start_a,)* $mid_a], [$($end_a),*], [$($start_b,)* $mid_b], [$($end_b),*], $lower, $diag, $upper}
    };
    // base case, last element on the diagonal
    (@diag[$($start_a:tt),*], [$last_a:tt], [$($start_b:tt),*], [$last_b:tt], $lower:tt, $diag:tt, $upper:tt) => {
        $($lower!($last_a, $start_b);)*
        $diag!($last_a, $last_b);
    };

    // friendly public interface
    // execute submacro on each member of the cartesian product of a and b
    ([$($a:tt)*], [$($b:tt)*], $submacro:tt) =>
        (cartesian!{@[] [$($a)*,] [$($b)*,] [$($b)*,] $submacro});
    ([$($a:tt)*], $submacro:tt) =>
        (cartesian!{@[] [$($a)*,] [$($a)*,] [$($a)*,] $submacro});
    // execute lower, diag and upper on the respective regions of the cartesian product of a and b
    ([$($a:tt)*], [$($b:tt)*], $lower:tt, $diag:tt, $upper:tt) =>
        (cartesian!{@diag[], [$($a)*], [], [$($b)*], $lower, $diag, $upper});
    ([$($a:tt)*], $lower:tt, $diag:tt, $upper:tt) =>
        (cartesian!{@diag[], [$($a)*], [], [$($a)*], $lower, $diag, $upper});
}

/// Fallible casting where the casted value is equal to the original value.
/// Casting fails for any value not between Self::MIN_CONSECUTIVE and Self::MAX_CONSECUTIVE.
pub trait ExactIntCast<TI>: Sized + ExactIntBounds {
    fn exact_int_cast(v: TI) -> Fallible<Self>;
}
macro_rules! impl_exact_int_cast_from {
    ($ti:ty, $to:ty) => (impl ExactIntCast<$ti> for $to {
        #[inline]
        fn exact_int_cast(v: $ti) -> Fallible<Self> {Ok(From::from(v))}
    })
}
macro_rules! impl_exact_int_cast_try_from {
    ($ti:ty, $to:ty) => (impl ExactIntCast<$ti> for $to {
        fn exact_int_cast(v: $ti) -> Fallible<Self> {
            TryFrom::try_from(v).map_err(|e| err!(FailedCast, "{:?}", e))
        }
    })
}
// top left
cartesian!{[u8, u16, u32, u64, u128], impl_exact_int_cast_try_from, impl_exact_int_cast_from, impl_exact_int_cast_from}
// top right
cartesian!([u8, u16, u32, u64, u128], [i8, i16, i32, i64, i128], impl_exact_int_cast_try_from, impl_exact_int_cast_try_from, impl_exact_int_cast_from);
// bottom left
cartesian!([i8, i16, i32, i64, i128], [u8, u16, u32, u64, u128], impl_exact_int_cast_try_from);
// bottom right
cartesian!{[i8, i16, i32, i64, i128], impl_exact_int_cast_try_from, impl_exact_int_cast_from, impl_exact_int_cast_from}

macro_rules! impl_exact_int_cast_int_float {
    ($int:ty, $float:ty) => (impl ExactIntCast<$int> for $float {
        fn exact_int_cast(v_int: $int) -> Fallible<Self> {
            let v_float = v_int as $float;
            if <$float>::MIN_CONSECUTIVE > v_float || <$float>::MAX_CONSECUTIVE < v_float {
                fallible!(FailedCast, "exact_int_cast: integer is outside of consecutive integer bounds and may be subject to rounding")
            } else {
                Ok(v_float)
            }
        }
    })
}

cartesian!([u8, u16, i8, i16], [f32, f64], impl_exact_int_cast_from);
cartesian!([u64, u128, i64, i128, usize], [f32, f64], impl_exact_int_cast_int_float);
impl_exact_int_cast_int_float!(u32, f32);
impl_exact_int_cast_from!(u32, f64);
impl_exact_int_cast_int_float!(i32, f32);
impl_exact_int_cast_from!(i32, f64);

cartesian!([usize], [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128], impl_exact_int_cast_try_from);

/// Fallible casting where the casted value rounds towards infinity.
/// This preserves the invariant that the casted value is gte the original value.
/// For example, casting a 128_u8 to i8 doesn't saturate to i8::MAX (127), it errors.
pub trait InfCast<TI>: Sized {
    fn inf_cast(v: TI) -> Fallible<Self>;
}

macro_rules! impl_inf_cast_exact {
    ($ti:ty, $to:ty) => (impl InfCast<$ti> for $to {
        fn inf_cast(v: $ti) -> Fallible<Self> { ExactIntCast::exact_int_cast(v) }
    })
}
cartesian!([u8, u16, u32, u64, u128, i8, i16, i32, i64, i128], impl_inf_cast_exact);


macro_rules! impl_inf_cast_from {
    ($ti:ty, $to:ty) => (impl InfCast<$ti> for $to {
        #[inline]
        fn inf_cast(v: $ti) -> Fallible<Self> { Ok(From::from(v)) }
    })
}

pub trait ExactIntBounds {
    const MAX_CONSECUTIVE: Self;
    const MIN_CONSECUTIVE: Self;
}
macro_rules! impl_exact_int_bounds {
    ($($ty:ty),*) => ($(impl ExactIntBounds for $ty {
        const MAX_CONSECUTIVE: Self = Self::MAX;
        const MIN_CONSECUTIVE: Self = Self::MIN;
    })*)
}
impl_exact_int_bounds!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize);
impl ExactIntBounds for f64 {
    const MAX_CONSECUTIVE: Self = 9_007_199_254_740_992.0;
    const MIN_CONSECUTIVE: Self = -9_007_199_254_740_992.0;
}
impl ExactIntBounds for f32 {
    const MAX_CONSECUTIVE: Self = 16_777_216.0;
    const MIN_CONSECUTIVE: Self = -16_777_216.0;
}

macro_rules! impl_inf_cast_int_float {
    ($int:ty, $float:ty) => (impl InfCast<$int> for $float {
        fn inf_cast(v_int: $int) -> Fallible<Self> {
            let v_float = v_int as $float;
            Ok(if v_int > v_float as $int { <$float>::from_bits(v_float.to_bits() + 1) } else { v_float })
        }
    })
}

cartesian!([u8, u16, i8, i16], [f32, f64], impl_inf_cast_from);
cartesian!([u64, u128, i64, i128], [f32, f64], impl_inf_cast_int_float);
impl_inf_cast_int_float!(u32, f32);
impl_inf_cast_from!(u32, f64);
impl_inf_cast_int_float!(i32, f32);
impl_inf_cast_from!(i32, f64);

impl_inf_cast_from!(f32, f32);
impl_inf_cast_from!(f32, f64);
impl InfCast<f64> for f32 {
    fn inf_cast(vf64: f64) -> Fallible<Self> {
        let vf32 = vf64 as f32;
        Ok(if vf64 > vf32 as f64 { f32::from_bits(vf32.to_bits() + 1) } else { vf32 })
    }
}
impl_inf_cast_from!(f64, f64);

macro_rules! impl_inf_cast_float_int {
    ($ti:ty, $to:ty) => (impl InfCast<$ti> for $to {
        fn inf_cast(mut v: $ti) -> Fallible<Self> {
            v = v.ceil();
            if Self::MIN as $ti > v || Self::MAX as $ti < v {
                fallible!(FailedCast, "Failed to cast float to int. Float value is outside of range.")
            } else {
                Ok(v as Self)
            }
        }
    })
}
cartesian!([f32, f64], [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128], impl_inf_cast_float_int);

#[cfg(test)]
mod test_inf_cast {
    use crate::traits::InfCast;

    #[allow(dead_code)]
    enum Diff { Equal, Prev, Next, Less, Greater }

    fn check_rounded_cast(input: f64, diff: Diff) {
        let casted = f32::inf_cast(input).unwrap() as f64;
        if input.is_nan() { assert!(casted.is_nan()); return }

        let error = match diff {
            Diff::Equal => (casted != input)
                .then(|| "casted value must be equal to input"),
            Diff::Greater => (casted <= input)
                .then(|| "casted value must be greater than input value"),
            Diff::Less => (casted >= input)
                .then(|| "casted value must be less than input value"),
            Diff::Next => (f64::from_bits(input.to_bits() + 1) != casted)
                .then(|| "casted must be one step greater than input"),
            Diff::Prev => (f64::from_bits(input.to_bits() - 1) != casted)
                .then(|| "casted must be one step less than input"),
        };
        if let Some(message) = error {
            println!("bits      {:064b}", input.to_bits());
            println!("input     {}", input);
            println!("output    {}", casted);
            panic!("{}", message)
        }
    }

    #[test]
    // ignored test because it can take a while to run
    #[ignore]
    fn test_f64_f32() {
        check_rounded_cast(0., Diff::Equal);
        // check that the f64 one step above zero casts to a value that is greater
        check_rounded_cast(f64::MIN_POSITIVE, Diff::Greater);
        // check that the f64 one step below 2 casts to exactly 2
        check_rounded_cast(1.9999999999999998, Diff::Next);
        // for each non-negative, nonzero f32
        for u32_bits in 1..u32::MAX / 2 {
            let f64_value = f32::from_bits(u32_bits) as f64;
            let u64_bits = f64_value.to_bits();

            if u32_bits % 100_000_000 == 0 {
                println!("checkpoint every 300 million tests: {}", f64_value);
            }
            // check that the f64 equivalent to the current f32 casts to a value that is equivalent
            check_rounded_cast(f64_value, Diff::Equal);
            // check that the f64 one step below the f64 equivalent to the current f32 casts to a value that is one step greater
            check_rounded_cast(f64::from_bits(u64_bits - 1), Diff::Next);
            // check that the f64 one step above the f64 equivalent to the current f32 casts to a value that is greater
            check_rounded_cast(f64::from_bits(u64_bits + 1), Diff::Greater);
        }
    }
}
pub trait RoundCast<TI>: Sized {
    fn round_cast(v: TI) -> Fallible<Self>;
}
macro_rules! impl_round_cast_num {
    ($TI:ty, $TO:ty) => {
        impl RoundCast<$TI> for $TO {
            fn round_cast(v: $TI) -> Fallible<Self> {
                <$TO as NumCast>::from(v).ok_or_else(|| err!(FailedCast))
            }
        }
    }
}

macro_rules! impl_round_cast_self_string_bool {
    ($T:ty, $_T:ty) => {
        impl RoundCast<$T> for $T {
            fn round_cast(v: $T) -> Fallible<Self> {Ok(v)}
        }
        impl RoundCast<bool> for $T {
            fn round_cast(v: bool) -> Fallible<Self> {
                Ok(if v {Self::one()} else {Self::zero()})
            }
        }
        impl RoundCast<$T> for bool {
            fn round_cast(v: $T) -> Fallible<Self> {Ok(v.is_zero())}
        }
        impl RoundCast<String> for $T {
            fn round_cast(v: String) -> Fallible<Self> {
                v.parse::<$T>().map_err(|_e| err!(FailedCast))
            }
        }
        impl RoundCast<$T> for String {
            fn round_cast(v: $T) -> Fallible<Self> {Ok(v.to_string())}
        }
    }
}
cartesian!{[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64], impl_round_cast_num, impl_round_cast_self_string_bool, impl_round_cast_num}

// final four casts among bool and string
impl RoundCast<bool> for bool { fn round_cast(v: bool) -> Fallible<Self> {Ok(v)} }
impl RoundCast<String> for String { fn round_cast(v: String) -> Fallible<Self> {Ok(v)} }
impl RoundCast<String> for bool { fn round_cast(v: String) -> Fallible<Self> { Ok(!v.is_empty()) } }
impl RoundCast<bool> for String { fn round_cast(v: bool) -> Fallible<Self> { Ok(v.to_string()) } }

pub trait CheckNull { fn is_null(&self) -> bool; }
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
        } else {true}
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


/// Performs addition that saturates at the numeric bounds instead of overflowing.
pub trait SaturatingAdd: Sized {
    /// Saturating addition. Computes `self + v`,
    /// saturating at the relevant high or low boundary of the type.
    fn saturating_add(&self, v: &Self) -> Self;
}
/// Performs multiplication that returns none if overflowing.
pub trait CheckedMul: Sized {
    /// Checked multiplication.
    /// Returns `Some(self * v)` if the result does not overflow, else `None`
    fn checked_mul(&self, v: &Self) -> Option<Self>;
}
/// Performs subtraction that returns none if overflowing.
pub trait CheckedSub: Sized {
    /// Checked subtraction.
    /// Returns `Some(self - v)` if the result does not overflow, else `None`
    fn checked_sub(&self, v: &Self) -> Option<Self>;
}
macro_rules! impl_math_delegation {
    ($($t:ty),+) => {
        $(impl SaturatingAdd for $t {
            #[inline]
            fn saturating_add(&self, v: &Self) -> Self {
                <$t>::saturating_add(*self, *v)
            }
        })+
        $(impl CheckedMul for $t {
            #[inline]
            fn checked_mul(&self, v: &Self) -> Option<Self> {
                <$t>::checked_mul(*self, *v)
            }
        })+
        $(impl CheckedSub for $t {
            #[inline]
            fn checked_sub(&self, v: &Self) -> Option<Self> {
                <$t>::checked_sub(*self, *v)
            }
        })+
    };
}
impl_math_delegation!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
macro_rules! impl_math_float {
    ($($t:ty),+) => {
        $(impl SaturatingAdd for $t {
            fn saturating_add(&self, v: &Self) -> Self {
                (self + v).clamp(<$t>::MIN, <$t>::MAX)
            }
        })+
        $(impl CheckedMul for $t {
            fn checked_mul(&self, v: &Self) -> Option<Self> {
                let y = self * v;
                y.is_finite().then(|| y)
            }
        })+
        $(impl CheckedSub for $t {
            fn checked_sub(&self, v: &Self) -> Option<Self> {
                let y = self - v;
                y.is_finite().then(|| y)
            }
        })+
    }
}
impl_math_float!(f32, f64);

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

/// TotalOrd is well-defined on types that are Ord on their non-null values.
/// The framework provides a way to ensure values are non-null at runtime.
/// This trait should only be used when the framework can rely on these assurances.
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


pub trait FloatBits: Sized {
    type Bits: Copy + One + Zero + Eq
    + Shr<Output=Self::Bits> + Shl<Output=Self::Bits>
    + BitAnd<Output=Self::Bits> + Sub<Output=Self::Bits>;
    const EXPONENT_BITS: Self::Bits;
    const MANTISSA_BITS: Self::Bits;
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

impl FloatBits for f64 {
    type Bits = u64;
    const EXPONENT_BITS: u64 = 11;
    const MANTISSA_BITS: u64 = 52;
    const EXPONENT_PROB: u64 = 1022;
    fn to_bits(self) -> Self::Bits {self.to_bits()}
}
impl FloatBits for f32 {
    type Bits = u32;
    const EXPONENT_BITS: u32 = 8;
    const MANTISSA_BITS: u32 = 23;
    const EXPONENT_PROB: u32 = 126;
    fn to_bits(self) -> Self::Bits {self.to_bits()}
}
