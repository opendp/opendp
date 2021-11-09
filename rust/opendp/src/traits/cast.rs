use std::convert::TryFrom;

use num::{NumCast, One, Zero};
#[cfg(feature = "use-mpfr")]
use rug::Float;

use crate::error::Fallible;
use crate::traits::FloatBits;

// general overview of casters:
// https://docs.google.com/spreadsheets/d/1DJohiOI3EVHjwj8g4IEdFZVf7MMyFk_4oaSyjTfkO_0/edit?usp=sharing

/// Fallible casting where the casted value is equal to the original value.
/// Casting fails for any value not between Self::MIN_CONSECUTIVE and Self::MAX_CONSECUTIVE.
pub trait ExactIntCast<TI>: Sized + ExactIntBounds {
    fn exact_int_cast(v: TI) -> Fallible<Self>;
}

pub trait ExactIntBounds {
    const MAX_CONSECUTIVE: Self;
    const MIN_CONSECUTIVE: Self;
}

/// Fallible casting where the casted value rounds towards infinity.
/// This preserves the invariant that the casted value is gte the original value.
/// For example, casting a 128_u8 to i8 doesn't saturate to i8::MAX (127), it errors.
pub trait InfCast<TI>: Sized {
    fn inf_cast(v: TI) -> Fallible<Self>;
}

pub trait RoundCast<TI>: Sized {
    fn round_cast(v: TI) -> Fallible<Self>;
}

#[cfg(feature = "use-mpfr")]
pub trait CastInternalReal: FloatBits + Sized {
    // Number of digits in the mantissa.
    // MANTISSA_DIGITS == MANTISSA_BITS + 1 because of implicit bit
    const MANTISSA_DIGITS: u32;
    fn from_internal(v: Float) -> Self;
    fn into_internal(self) -> Float;
}

#[cfg(not(feature = "use-mpfr"))]
pub trait CastInternalReal: rand::distributions::uniform::SampleUniform + SampleGaussian {
    fn from_internal(v: Self) -> Self;
    fn into_internal(self) -> Self;
}


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


// TRAIT ExactIntCast
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
cartesian! {[u8, u16, u32, u64, u128], impl_exact_int_cast_try_from, impl_exact_int_cast_from, impl_exact_int_cast_from}
// top right
cartesian!([u8, u16, u32, u64, u128], [i8, i16, i32, i64, i128], impl_exact_int_cast_try_from, impl_exact_int_cast_try_from, impl_exact_int_cast_from);
// bottom left
cartesian!([i8, i16, i32, i64, i128], [u8, u16, u32, u64, u128], impl_exact_int_cast_try_from);
// bottom right
cartesian! {[i8, i16, i32, i64, i128], impl_exact_int_cast_try_from, impl_exact_int_cast_from, impl_exact_int_cast_from}

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

// usize conversions
cartesian!([usize], [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128], impl_exact_int_cast_try_from);
cartesian!([u8, u16, u32, u64, u128, i8, i16, i32, i64, i128], [usize], impl_exact_int_cast_try_from);
impl_exact_int_cast_from!(usize, usize);


// TRAIT InfCast
macro_rules! impl_inf_cast_exact {
    ($ti:ty, $to:ty) => (impl InfCast<$ti> for $to {
        fn inf_cast(v: $ti) -> Fallible<Self> { ExactIntCast::exact_int_cast(v) }
    })
}
cartesian!([u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize], impl_inf_cast_exact);


macro_rules! impl_inf_cast_from {
    ($ti:ty, $to:ty) => (impl InfCast<$ti> for $to {
        #[inline]
        fn inf_cast(v: $ti) -> Fallible<Self> { Ok(From::from(v)) }
    })
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
    ($int:ty, $float:ty) => (
        #[cfg(feature="use-mpfr")]
        impl InfCast<$int> for $float {
            fn inf_cast(v_int: $int) -> Fallible<Self> {
                use rug::{Float, float::Round};
                let float = Float::with_val_round(<$float>::MANTISSA_DIGITS, v_int, Round::Up).0;
                Ok(<$float>::from_internal(float))
            }
        }
        #[cfg(not(feature="use-mpfr"))]
        impl InfCast<$int> for $float {
            fn inf_cast(v_int: $int) -> Fallible<Self> {
                <$float>::round_cast(v_int)
            }
        }
    )
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
        if vf64.is_nan() { return Ok(f32::NAN) }
        // cast with rounding towards nearest, ties toward even
        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics
        let vf32 = vf64 as f32;

        // if nearest was toward -inf, then perturb one step towards inf
        // +/- zero always evaluates to false
        if vf64 > vf32 as f64 {
            return Ok(f32::from_bits(if vf32.is_sign_negative() {
                vf32.to_bits() - 1
            } else {
                vf32.to_bits() + 1
            }))
        }
        Ok(vf32)
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
        if input.is_nan() {
            assert!(casted.is_nan());
            return
        }

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


// TRAIT RoundCast
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
cartesian! {[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64], impl_round_cast_num, impl_round_cast_self_string_bool, impl_round_cast_num}

// final four casts among bool and string
impl RoundCast<bool> for bool { fn round_cast(v: bool) -> Fallible<Self> { Ok(v) } }

impl RoundCast<String> for String { fn round_cast(v: String) -> Fallible<Self> { Ok(v) } }

impl RoundCast<String> for bool { fn round_cast(v: String) -> Fallible<Self> { Ok(!v.is_empty()) } }

impl RoundCast<bool> for String { fn round_cast(v: bool) -> Fallible<Self> { Ok(v.to_string()) } }


#[cfg(feature = "use-mpfr")]
impl CastInternalReal for f64 {
    const MANTISSA_DIGITS: u32 = Self::MANTISSA_DIGITS;
    fn from_internal(v: Float) -> Self { v.to_f64() }
    fn into_internal(self) -> Float { rug::Float::with_val(Self::MANTISSA_DIGITS, self) }
}

#[cfg(feature = "use-mpfr")]
impl CastInternalReal for f32 {
    const MANTISSA_DIGITS: u32 = Self::MANTISSA_DIGITS;
    fn from_internal(v: Float) -> Self { v.to_f32() }
    fn into_internal(self) -> Float { rug::Float::with_val(Self::MANTISSA_DIGITS, self) }
}

#[cfg(not(feature = "use-mpfr"))]
impl CastInternalReal for f64 {
    const MANTISSA_DIGITS: u32 = Self::MANTISSA_DIGITS;
    fn from_internal(v: f64) -> Self { v }
    fn into_internal(self) -> Self { self }
}

#[cfg(not(feature = "use-mpfr"))]
impl CastInternalReal for f32 {
    const MANTISSA_DIGITS: u32 = Self::MANTISSA_DIGITS;
    fn from_internal(v: f32) -> Self { v }
    fn into_internal(self) -> Self { self }
}
