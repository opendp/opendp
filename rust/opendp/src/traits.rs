use std::ops::{Div, Mul, Sub};

use num::{NumCast, One, ToPrimitive, Zero};

use crate::error::Fallible;

pub trait CheckContinuous { fn is_continuous() -> bool; }
pub trait Ceil: Clone { fn ceil(self) -> Self; }
macro_rules! impl_is_continuous {
    ($($ty:ty),+) => {
        $(
            impl Ceil for $ty {
                #[inline]
                fn ceil(self) -> $ty { self.ceil() }
            }
            impl CheckContinuous for $ty {
                #[inline]
                fn is_continuous() -> bool {true}
            }
        )+
    }
}
macro_rules! impl_is_not_continuous {
    ($($ty:ty),+) => {
        $(
            impl Ceil for $ty {
                #[inline]
                fn ceil(self) -> $ty { self }
            }
            impl CheckContinuous for $ty {
                #[inline]
                fn is_continuous() -> bool {false}
            }
        )+
    }
}
impl_is_continuous!(f32, f64);
impl_is_not_continuous!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize, usize);

/// A type that can be used as a stability or privacy constant to scale a distance
pub trait DistanceConstant: 'static + Clone + DistanceCast + Div<Output=Self> + Mul<Output=Self> + PartialOrd {}
impl<T: 'static + Clone + DistanceCast + Div<Output=Self> + Mul<Output=Self> + PartialOrd> DistanceConstant for T {}

// TODO: Maybe this should be renamed to something more specific to budgeting, and add negative checks? -Mike
pub trait FallibleSub<Rhs = Self> {
    type Output;
    fn sub(self, rhs: Rhs) -> Fallible<Self::Output>;
}

macro_rules! impl_fallible_sub {
    ($($ty:ty),+) => ($(
        impl<Rhs> FallibleSub<Rhs> for $ty where $ty: Sub<Rhs, Output=$ty> {
            type Output = $ty;
            fn sub(self, rhs: Rhs) -> Fallible<Self::Output> {
                Ok(self - rhs)
            }
        }
    )+)
}
impl_fallible_sub!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl<T0, T1, Rhs0, Rhs1> FallibleSub<(Rhs0, Rhs1)> for (T0, T1) where T0: Sub<Rhs0>, T1: Sub<Rhs1> {
    type Output = (T0::Output, T1::Output);
    fn sub(self, rhs: (Rhs0, Rhs1)) -> Fallible<Self::Output> {
        Ok((self.0 - rhs.0, self.1 - rhs.1))
    }
}

impl<'a, T0, T1, Rhs0, Rhs1> FallibleSub<&'a (Rhs0, Rhs1)> for (T0, T1) where T0: Sub<&'a Rhs0>, T1: Sub<&'a Rhs1> {
    type Output = (T0::Output, T1::Output);
    fn sub(self, rhs: &'a (Rhs0, Rhs1)) -> Fallible<Self::Output> {
        Ok((self.0 - &rhs.0, self.1 - &rhs.1))
    }
}

/// A type that can be used as a measure distance.
pub trait MeasureDistance: PartialOrd + for<'a> FallibleSub<&'a Self, Output=Self> {}
impl<T> MeasureDistance for T where T: PartialOrd + for<'a> FallibleSub<&'a Self, Output=Self> {}

/// A type that can be used as a metric distance.
pub trait MetricDistance: PartialOrd {}
impl<T> MetricDistance for T where T: PartialOrd {}

// include Ceil on QO to avoid requiring as an additional trait bound in all downstream code
pub trait DistanceCast: NumCast + Ceil + CheckContinuous {
    fn distance_cast<T: ToPrimitive + Ceil>(n: T) -> Fallible<Self>;
}

impl<QO: ToPrimitive + NumCast + CheckContinuous + Ceil> DistanceCast for QO {
    fn distance_cast<QI: ToPrimitive + Ceil>(v: QI) -> Fallible<QO> {
        // round away from zero when losing precision
        QO::from(if QO::is_continuous() { v } else { v.ceil() }).ok_or_else(|| err!(FailedCast))
    }
}


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
pub trait CastFrom<TI>: Sized {
    fn cast(v: TI) -> Fallible<Self>;
}
macro_rules! impl_num_cast {
    ($TI:ty, $TO:ty) => {
        impl CastFrom<$TI> for $TO {
            fn cast(v: $TI) -> Fallible<Self> {
                <$TO as NumCast>::from(v).ok_or_else(|| err!(FailedCast))
            }
        }
    }
}
macro_rules! impl_self_cast {
    ($T:ty) => {
        impl CastFrom<$T> for $T {
            fn cast(v: $T) -> Fallible<Self> {
                Ok(v)
            }
        }
    }
}
macro_rules! impl_bool_cast {
    ($T:ty) => {
        impl CastFrom<bool> for $T {
            fn cast(v: bool) -> Fallible<Self> {
                Ok(if v {Self::one()} else {Self::zero()})
            }
        }
        impl CastFrom<$T> for bool {
            fn cast(v: $T) -> Fallible<Self> {
                Ok(v.is_zero())
            }
        }
    }
}
macro_rules! impl_string_cast {
    ($T:ty) => {
        impl CastFrom<String> for $T {
            fn cast(v: String) -> Fallible<Self> {
                v.parse::<$T>().map_err(|_e| err!(FailedCast))
            }
        }
        impl CastFrom<$T> for String {
            fn cast(v: $T) -> Fallible<Self> {
                Ok(v.to_string())
            }
        }
    }
}
macro_rules! impl_for_each {
    ([];[$first:ty, $($end:ty),*]) => {
        impl_self_cast!($first);
        impl_bool_cast!($first);
        impl_string_cast!($first);
        $(impl_num_cast!($first, $end);)*

        impl_for_each!{[$first];[$($end),*]}
    };
    ([$($start:ty),*];[$mid:ty, $($end:ty),*]) => {
        $(impl_num_cast!($mid, $start);)*
        impl_self_cast!($mid);
        impl_bool_cast!($mid);
        impl_string_cast!($mid);
        $(impl_num_cast!($mid, $end);)*

        impl_for_each!{[$($start),*, $mid];[$($end),*]}
    };
    ([$($start:ty),*];[$last:ty]) => {
        impl_self_cast!($last);
        impl_bool_cast!($last);
        impl_string_cast!($last);
        $(impl_num_cast!($last, $start);)*
    };
}
impl_for_each!{[];[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64]}

// final four casts among bool and string
impl_self_cast!(bool);
impl_self_cast!(String);
impl CastFrom<String> for bool {
    fn cast(v: String) -> Fallible<Self> {
        Ok(!v.is_empty())
    }
}
impl CastFrom<bool> for String {
    fn cast(v: bool) -> Fallible<Self> {
        Ok(v.to_string())
    }
}
