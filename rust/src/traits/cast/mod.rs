use std::convert::TryFrom;

use dashu::{
    base::Approximation,
    float::{
        round::{
            mode::{Down, HalfEven, Up},
            Round, Rounding,
        },
        FBig,
    },
    integer::{IBig, UBig},
    rational::RBig,
};
use num::{NumCast, One, Zero};

use crate::error::Fallible;

// general overview of casters:
// https://docs.google.com/spreadsheets/d/1DJohiOI3EVHjwj8g4IEdFZVf7MMyFk_4oaSyjTfkO_0/edit?usp=sharing

/// Fallible casting where the casted value is exactly equal to the original value.
pub trait ExactIntCast<TI>: Sized + ExactIntBounds {
    /// # Proof Definition
    /// For any `v` of type `TI`, `Self::exact_int_cast(value)` either
    /// returns `Err(e)` if `v` is smaller than `Self::MIN_CONSECUTIVE` or greater than `Self::MAX_CONSECUTIVE`,
    /// or `Ok(out)` where $out = v$.
    fn exact_int_cast(v: TI) -> Fallible<Self>;
}

/// Consts representing the maximum and minimum finite consecutive values.
///
/// This is also implemented for floats,
/// as neighboring floating point values may differ by more than 1 when the mantissa is exhausted.
pub trait ExactIntBounds {
    /// # Proof Definition
    /// `Self::MAX_CONSECUTIVE` is the largest integer-consecutive finite value that can be represented by `Self`.
    const MAX_CONSECUTIVE: Self;
    /// # Proof Definition
    /// `Self::MIN_CONSECUTIVE` is the smallest integer-consecutive finite value that can be represented by `Self`.
    const MIN_CONSECUTIVE: Self;
}

/// Fallible casting where the casted value rounds towards infinity.
///
/// This preserves the invariant that the casted value is gte the original value.
/// For example, casting a 128_u8 to i8 doesn't saturate to i8::MAX (127), it errors.
pub trait InfCast<TI>: Sized {
    /// # Proof Definition
    /// For any `v` of type `TI`, `Self::inf_cast(value)` either returns `Err(e)`,
    /// or `Ok(out)` where $out \ge v$.
    fn inf_cast(v: TI) -> Fallible<Self>;
    /// # Proof Definition
    /// For any `v` of type `TI`, `Self::inf_cast(value)` either returns `Err(e)`,
    /// or `Ok(out)` where $out \le v$.
    fn neg_inf_cast(v: TI) -> Fallible<Self>;
}

/// Fallible casting where the casted value is rounded to nearest.
pub trait RoundCast<TI>: Sized {
    /// # Proof Definition
    /// For any `v` of type `TI`, `Self::inf_cast(v)` either returns `Err(e)`,
    /// or `Ok(out)` where $out = argmin_{x \in TI} |x - v|$.
    fn round_cast(v: TI) -> Fallible<Self>;
}

/// Fallible casting where the casted value saturates.
pub trait SaturatingCast<TI>: Sized {
    /// # Proof Definition
    /// For any `v` of type `TI`, `Self::saturating_cast(v)` either returns `Err(e)`,
    /// or `Ok(out)` where $out = clamp(v, TI::MIN, TI::MAX)$.
    fn saturating_cast(v: TI) -> Self;
}

/// Casting between floating-point and rational values.
pub trait CastInternalRational {
    /// # Proof Definition
    /// For any [`RBig`] `v`, return `out`, the nearest representable value of type `Self`.
    /// `out` may saturate to +/- infinity.
    fn from_rational(v: RBig) -> Self;
    /// # Proof Definition
    /// For any `self` of type `Self`, either return
    /// `Err(e)` if `self` is not finite, or
    /// `Ok(out)`, where `out` is a [`RBig`] that exactly represents `self`.
    fn into_rational(self) -> Fallible<RBig>;
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
pub(crate) use cartesian;

// TRAIT ExactIntCast
macro_rules! impl_exact_int_cast_from {
    ($ti:ty, $to:ty) => {
        impl ExactIntCast<$ti> for $to {
            #[inline]
            fn exact_int_cast(v: $ti) -> Fallible<Self> {
                Ok(From::from(v))
            }
        }
    };
}
macro_rules! impl_exact_int_cast_try_from {
    ($ti:ty, $to:ty) => {
        impl ExactIntCast<$ti> for $to {
            fn exact_int_cast(v: $ti) -> Fallible<Self> {
                TryFrom::try_from(v).map_err(|e| err!(FailedCast, "{:?}", e))
            }
        }
    };
}
// top left
cartesian! {[u8, u16, u32, u64, u128], impl_exact_int_cast_try_from, impl_exact_int_cast_from, impl_exact_int_cast_from}
// top right
cartesian!(
    [u8, u16, u32, u64, u128],
    [i8, i16, i32, i64, i128],
    impl_exact_int_cast_try_from,
    impl_exact_int_cast_try_from,
    impl_exact_int_cast_from
);
// bottom left
cartesian!(
    [i8, i16, i32, i64, i128],
    [u8, u16, u32, u64, u128],
    impl_exact_int_cast_try_from
);
// bottom right
cartesian! {[i8, i16, i32, i64, i128], impl_exact_int_cast_try_from, impl_exact_int_cast_from, impl_exact_int_cast_from}

macro_rules! impl_exact_int_cast_int_float {
    ($int:ty, $float:ty) => (impl ExactIntCast<$int> for $float {
        fn exact_int_cast(v_int: $int) -> Fallible<Self> {
            let v_float = v_int as $float;
            if !(<$float>::MIN_CONSECUTIVE..<$float>::MAX_CONSECUTIVE).contains(&v_float) {
                fallible!(FailedCast, "exact_int_cast: integer is outside of consecutive integer bounds and may be subject to rounding")
            } else {
                Ok(v_float)
            }
        }
    })
}

cartesian!([u8, u16, i8, i16], [f32, f64], impl_exact_int_cast_from);
cartesian!(
    [u64, u128, i64, i128, usize, isize],
    [f32, f64],
    impl_exact_int_cast_int_float
);
impl_exact_int_cast_int_float!(u32, f32);
impl_exact_int_cast_from!(u32, f64);
impl_exact_int_cast_int_float!(i32, f32);
impl_exact_int_cast_from!(i32, f64);

// usize conversions
cartesian!(
    [usize, isize],
    [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128],
    impl_exact_int_cast_try_from
);
cartesian!(
    [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128],
    [usize, isize],
    impl_exact_int_cast_try_from
);
impl_exact_int_cast_from!(usize, usize);
impl_exact_int_cast_from!(isize, isize);
impl_exact_int_cast_try_from!(usize, isize);
impl_exact_int_cast_try_from!(isize, usize);

// TRAIT InfCast
macro_rules! impl_inf_cast_exact {
    ($ti:ty, $to:ty) => {
        impl InfCast<$ti> for $to {
            fn inf_cast(v: $ti) -> Fallible<Self> {
                ExactIntCast::exact_int_cast(v)
            }
            fn neg_inf_cast(v: $ti) -> Fallible<Self> {
                ExactIntCast::exact_int_cast(v)
            }
        }
    };
}
cartesian!(
    [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize],
    impl_inf_cast_exact
);

macro_rules! impl_inf_cast_from {
    ($ti:ty, $to:ty) => {
        impl InfCast<$ti> for $to {
            #[inline]
            fn inf_cast(v: $ti) -> Fallible<Self> {
                Ok(From::from(v))
            }
            fn neg_inf_cast(v: $ti) -> Fallible<Self> {
                Ok(From::from(v))
            }
        }
    };
}

macro_rules! impl_exact_int_bounds {
    ($($ty:ty),*) => ($(impl ExactIntBounds for $ty {
        const MAX_CONSECUTIVE: Self = Self::MAX;
        const MIN_CONSECUTIVE: Self = Self::MIN;
    })*)
}
impl_exact_int_bounds!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
impl ExactIntBounds for f64 {
    const MAX_CONSECUTIVE: Self = 9_007_199_254_740_992.0;
    const MIN_CONSECUTIVE: Self = -9_007_199_254_740_992.0;
}

impl ExactIntBounds for f32 {
    const MAX_CONSECUTIVE: Self = 16_777_216.0;
    const MIN_CONSECUTIVE: Self = -16_777_216.0;
}

pub(crate) trait NextFloat {
    // naming has a trailing underscore so as not to conflict with stabilization of the corresponding methods in Rust STD
    fn next_up_(self) -> Self;
    fn next_down_(self) -> Self;
}

impl NextFloat for f64 {
    /// Returns the least number greater than `self`.
    ///
    /// Taken from the unstable Rust compiler implementation.
    /// https://github.com/rust-lang/rust/blob/eb33b43bab08223fa6b46abacc1e95e859fe375d/library/core/src/num/f64.rs#L763-L810
    fn next_up_(self) -> Self {
        // Some targets violate Rust's assumption of IEEE semantics, e.g. by flushing
        // denormals to zero. This is in general unsound and unsupported, but here
        // we do our best to still produce the correct result on such targets.
        const TINY_BITS: u64 = 0x1; // Smallest positive f64.
        const CLEAR_SIGN_MASK: u64 = 0x7fff_ffff_ffff_ffff;

        let bits = self.to_bits();
        if self.is_nan() || bits == Self::INFINITY.to_bits() {
            return self;
        }

        let abs = bits & CLEAR_SIGN_MASK;
        let next_bits = if abs == 0 {
            TINY_BITS
        } else if bits == abs {
            bits + 1
        } else {
            bits - 1
        };
        Self::from_bits(next_bits)
    }

    /// Returns the greatest number less than `self`.
    ///
    /// Taken from the unstable Rust compiler implementation.
    /// https://github.com/rust-lang/rust/blob/eb33b43bab08223fa6b46abacc1e95e859fe375d/library/core/src/num/f64.rs#L812-L859
    fn next_down_(self) -> Self {
        // Some targets violate Rust's assumption of IEEE semantics, e.g. by flushing
        // denormals to zero. This is in general unsound and unsupported, but here
        // we do our best to still produce the correct result on such targets.
        const NEG_TINY_BITS: u64 = 0x8000_0000_0000_0001; // Smallest (in magnitude) negative f64.
        const CLEAR_SIGN_MASK: u64 = 0x7fff_ffff_ffff_ffff;

        let bits = self.to_bits();
        if self.is_nan() || bits == Self::NEG_INFINITY.to_bits() {
            return self;
        }

        let abs = bits & CLEAR_SIGN_MASK;
        let next_bits = if abs == 0 {
            NEG_TINY_BITS
        } else if bits == abs {
            bits - 1
        } else {
            bits + 1
        };
        Self::from_bits(next_bits)
    }
}
impl NextFloat for f32 {
    /// Returns the least number greater than `self`.
    ///
    /// Taken from the unstable Rust compiler implementation.
    /// https://github.com/rust-lang/rust/blob/eb33b43bab08223fa6b46abacc1e95e859fe375d/library/core/src/num/f32.rs#L750-L797
    fn next_up_(self) -> Self {
        // Some targets violate Rust's assumption of IEEE semantics, e.g. by flushing
        // denormals to zero. This is in general unsound and unsupported, but here
        // we do our best to still produce the correct result on such targets.
        const TINY_BITS: u32 = 0x1; // Smallest positive f32.
        const CLEAR_SIGN_MASK: u32 = 0x7fff_ffff;

        let bits = self.to_bits();
        if self.is_nan() || bits == Self::INFINITY.to_bits() {
            return self;
        }

        let abs = bits & CLEAR_SIGN_MASK;
        let next_bits = if abs == 0 {
            TINY_BITS
        } else if bits == abs {
            bits + 1
        } else {
            bits - 1
        };
        Self::from_bits(next_bits)
    }

    /// Returns the greatest number less than `self`.
    ///
    /// Taken from the unstable Rust compiler implementation.
    /// https://github.com/rust-lang/rust/blob/eb33b43bab08223fa6b46abacc1e95e859fe375d/library/core/src/num/f32.rs#L799-L846
    fn next_down_(self) -> Self {
        // Some targets violate Rust's assumption of IEEE semantics, e.g. by flushing
        // denormals to zero. This is in general unsound and unsupported, but here
        // we do our best to still produce the correct result on such targets.
        const NEG_TINY_BITS: u32 = 0x8000_0001; // Smallest (in magnitude) negative f32.
        const CLEAR_SIGN_MASK: u32 = 0x7fff_ffff;

        let bits = self.to_bits();
        if self.is_nan() || bits == Self::NEG_INFINITY.to_bits() {
            return self;
        }

        let abs = bits & CLEAR_SIGN_MASK;
        let next_bits = if abs == 0 {
            NEG_TINY_BITS
        } else if bits == abs {
            bits - 1
        } else {
            bits + 1
        };
        Self::from_bits(next_bits)
    }
}

/// Wrapper around Dashu's FBig::to_f32 and FBig::to_f32 that guarantees correct rounding, even in edge cases
pub trait ToFloatRounded {
    fn to_f32_rounded(self) -> f32;
    fn to_f64_rounded(self) -> f64;
}

impl ToFloatRounded for FBig<Up> {
    fn to_f32_rounded(self) -> f32 {
        match self.to_f32() {
            Approximation::Exact(v) | Approximation::Inexact(v, Rounding::AddOne) => v,
            Approximation::Inexact(v, Rounding::SubOne)
            | Approximation::Inexact(v, Rounding::NoOp) => v.next_up_(),
        }
    }

    fn to_f64_rounded(self) -> f64 {
        match self.to_f64() {
            Approximation::Exact(v) | Approximation::Inexact(v, Rounding::AddOne) => v,
            Approximation::Inexact(v, Rounding::SubOne)
            | Approximation::Inexact(v, Rounding::NoOp) => v.next_up_(),
        }
    }
}

impl ToFloatRounded for FBig<Down> {
    fn to_f32_rounded(self) -> f32 {
        match self.to_f32() {
            Approximation::Exact(v) | Approximation::Inexact(v, Rounding::SubOne) => v,
            Approximation::Inexact(v, Rounding::AddOne)
            | Approximation::Inexact(v, Rounding::NoOp) => v.next_down_(),
        }
    }

    fn to_f64_rounded(self) -> f64 {
        match self.to_f64() {
            Approximation::Exact(v) | Approximation::Inexact(v, Rounding::SubOne) => v,
            Approximation::Inexact(v, Rounding::AddOne)
            | Approximation::Inexact(v, Rounding::NoOp) => v.next_down_(),
        }
    }
}

/// Convert from an FBig to a native type `Self` with controlled rounding
trait FromFBig<R: Round> {
    fn from_fbig(value: FBig<R>) -> Self;
}

impl<R: Round> FromFBig<R> for f32
where
    FBig<R>: ToFloatRounded,
{
    fn from_fbig(value: FBig<R>) -> Self {
        value.to_f32_rounded()
    }
}

impl<R: Round> FromFBig<R> for f64
where
    FBig<R>: ToFloatRounded,
{
    fn from_fbig(value: FBig<R>) -> Self {
        value.to_f64_rounded()
    }
}

macro_rules! impl_inf_cast_int_float {
    ($int:ty, $float:ty) => {
        impl InfCast<$int> for $float {
            fn inf_cast(v_int: $int) -> Fallible<Self> {
                Ok(<$float>::from_fbig(FBig::<Up>::from(IBig::from(v_int))))
            }
            fn neg_inf_cast(v_int: $int) -> Fallible<Self> {
                Ok(<$float>::from_fbig(FBig::<Down>::from(IBig::from(v_int))))
            }
        }
    };
}

cartesian!([u8, u16, i8, i16], [f32, f64], impl_inf_cast_from);
cartesian!(
    [u64, u128, i64, i128, usize, isize],
    [f32, f64],
    impl_inf_cast_int_float
);
impl_inf_cast_int_float!(u32, f32);
impl_inf_cast_from!(u32, f64);
impl_inf_cast_int_float!(i32, f32);
impl_inf_cast_from!(i32, f64);

impl_inf_cast_from!(f32, f32);
impl_inf_cast_from!(f32, f64);

impl InfCast<f64> for f32 {
    fn inf_cast(vf64: f64) -> Fallible<Self> {
        if vf64.is_nan() {
            return Ok(f32::NAN);
        }
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
            }));
        }
        Ok(vf32)
    }

    fn neg_inf_cast(vf64: f64) -> Fallible<Self> {
        if vf64.is_nan() {
            return Ok(f32::NAN);
        }
        // cast with rounding towards nearest, ties toward even
        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics
        let vf32 = vf64 as f32;

        // if nearest was toward inf, then perturb one step towards -inf
        // +/- zero always evaluates to false
        if vf64 < vf32 as f64 {
            return Ok(f32::from_bits(if vf32.is_sign_negative() {
                vf32.to_bits() + 1
            } else {
                vf32.to_bits() - 1
            }));
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

        fn neg_inf_cast(mut v: $ti) -> Fallible<Self> {
            v = v.floor();
            if Self::MIN as $ti > v || Self::MAX as $ti < v {
                fallible!(FailedCast, "Failed to cast float to int. Float value is outside of range.")
            } else {
                Ok(v as Self)
            }
        }
    })
}
cartesian!(
    [f32, f64],
    [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128],
    impl_inf_cast_float_int
);

#[cfg(test)]
mod test;

// TRAIT RoundCast
macro_rules! impl_round_cast_num {
    ($TI:ty, $TO:ty) => {
        impl RoundCast<$TI> for $TO {
            fn round_cast(v: $TI) -> Fallible<Self> {
                <$TO as NumCast>::from(v).ok_or_else(|| err!(FailedCast))
            }
        }
    };
}

macro_rules! impl_round_cast_self_string_bool {
    ($T:ty, $_T:ty) => {
        impl RoundCast<$T> for $T {
            fn round_cast(v: $T) -> Fallible<Self> {
                Ok(v)
            }
        }
        impl RoundCast<bool> for $T {
            fn round_cast(v: bool) -> Fallible<Self> {
                Ok(if v { Self::one() } else { Self::zero() })
            }
        }
        impl RoundCast<$T> for bool {
            fn round_cast(v: $T) -> Fallible<Self> {
                Ok(!v.is_zero())
            }
        }
        impl RoundCast<String> for $T {
            fn round_cast(v: String) -> Fallible<Self> {
                v.parse::<$T>().map_err(|_e| err!(FailedCast))
            }
        }
        impl RoundCast<$T> for String {
            fn round_cast(v: $T) -> Fallible<Self> {
                Ok(v.to_string())
            }
        }
    };
}
cartesian! {[u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64], impl_round_cast_num, impl_round_cast_self_string_bool, impl_round_cast_num}

// final four casts among bool and string
impl RoundCast<bool> for bool {
    fn round_cast(v: bool) -> Fallible<Self> {
        Ok(v)
    }
}

impl RoundCast<String> for String {
    fn round_cast(v: String) -> Fallible<Self> {
        Ok(v)
    }
}

impl RoundCast<String> for bool {
    fn round_cast(v: String) -> Fallible<Self> {
        Ok(!v.is_empty())
    }
}

impl RoundCast<bool> for String {
    fn round_cast(v: bool) -> Fallible<Self> {
        Ok(v.to_string())
    }
}

impl<R: Round> RoundCast<f32> for FBig<R> {
    fn round_cast(v: f32) -> Fallible<Self> {
        FBig::try_from(v).map_err(|_| err!(FailedCast, "found NaN"))
    }
}

impl<R: Round> RoundCast<f64> for FBig<R> {
    fn round_cast(v: f64) -> Fallible<Self> {
        FBig::try_from(v).map_err(|_| err!(FailedCast, "found NaN"))
    }
}

impl<R: Round> RoundCast<FBig<R>> for f32 {
    fn round_cast(v: FBig<R>) -> Fallible<Self> {
        Ok(v.with_rounding::<HalfEven>().to_f32().value())
    }
}

impl<R: Round> RoundCast<FBig<R>> for f64 {
    fn round_cast(v: FBig<R>) -> Fallible<Self> {
        Ok(v.with_rounding::<HalfEven>().to_f64().value())
    }
}

impl RoundCast<RBig> for f32 {
    fn round_cast(v: RBig) -> Fallible<Self> {
        Ok(v.to_f32().value())
    }
}

impl RoundCast<RBig> for f64 {
    fn round_cast(v: RBig) -> Fallible<Self> {
        Ok(v.to_f64().value())
    }
}

impl<R: Round> InfCast<f32> for FBig<R> {
    fn inf_cast(v: f32) -> Fallible<Self> {
        FBig::try_from(v).map_err(|_| err!(FailedCast, "found NaN"))
    }

    fn neg_inf_cast(v: f32) -> Fallible<Self> {
        FBig::try_from(v).map_err(|_| err!(FailedCast, "found NaN"))
    }
}

impl<R: Round> InfCast<f64> for FBig<R> {
    fn inf_cast(v: f64) -> Fallible<Self> {
        FBig::try_from(v).map_err(|_| err!(FailedCast, "found NaN"))
    }

    fn neg_inf_cast(v: f64) -> Fallible<Self> {
        FBig::try_from(v).map_err(|_| err!(FailedCast, "found NaN"))
    }
}

impl<R: Round> InfCast<FBig<R>> for f32 {
    fn inf_cast(v: FBig<R>) -> Fallible<Self> {
        Ok(v.with_rounding::<Up>().to_f32_rounded())
    }

    fn neg_inf_cast(v: FBig<R>) -> Fallible<Self> {
        Ok(v.with_rounding::<Down>().to_f32_rounded())
    }
}

impl<R: Round> InfCast<FBig<R>> for f64 {
    fn inf_cast(v: FBig<R>) -> Fallible<Self> {
        Ok(v.with_rounding::<Up>().to_f64_rounded())
    }

    fn neg_inf_cast(v: FBig<R>) -> Fallible<Self> {
        Ok(v.with_rounding::<Down>().to_f64_rounded())
    }
}

macro_rules! impl_saturating_cast_ubig_int {
    ($($T:ty)+) => {$(
        impl SaturatingCast<UBig> for $T {
            fn saturating_cast(v: UBig) -> Self {
                <$T>::try_from(v).unwrap_or(<$T>::MAX)
            }
        }
    )+}
}

impl_saturating_cast_ubig_int! {u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize}

macro_rules! impl_saturating_cast_ibig_uint {
    ($($T:ty)+) => {$(
        impl SaturatingCast<IBig> for $T {
            fn saturating_cast(v: IBig) -> Self {
                let positive = v > IBig::ZERO;
                <$T>::try_from(v).unwrap_or_else(|_| if positive { <$T>::MAX } else { <$T>::MIN })
            }
        }
    )+}
}
impl_saturating_cast_ibig_uint! {i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize}

macro_rules! impl_cast_internal_rational_float {
    ($ty:ty, $method:ident) => {
        impl CastInternalRational for $ty {
            fn from_rational(v: RBig) -> Self {
                v.$method().value()
            }
            fn into_rational(self) -> Fallible<RBig> {
                RBig::try_from(self).map_err(|_| {
                    err!(
                        FailedFunction,
                        "{} must be representable as a fraction",
                        self
                    )
                })
            }
        }
    };
}

impl_cast_internal_rational_float!(f32, to_f32);
impl_cast_internal_rational_float!(f64, to_f64);

macro_rules! impl_cast_internal_rational_int {
    ($($ty:ty)+) => {
        $(impl CastInternalRational for $ty {
            fn from_rational(v: RBig) -> Self {
                <$ty>::try_from(v.round())
                    .unwrap_or_else(|_| if v > RBig::ZERO { <$ty>::MAX } else { <$ty>::MIN })
            }
            fn into_rational(self) -> Fallible<RBig> {
                Ok(RBig::from(self))
            }
        })+
    };
}

impl_cast_internal_rational_int!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
