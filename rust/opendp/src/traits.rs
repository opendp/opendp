use num::{NumCast, ToPrimitive};
use crate::error::Fallible;

pub trait CheckContinuous { fn is_continuous() -> bool; }
pub trait Ceil : Copy { fn ceil(self) -> Self; }
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

// include Ceil on QO to avoid requiring as an additional trait bound in all downstream code
pub trait DistanceCast: NumCast + Ceil + CheckContinuous {
    fn cast<T: ToPrimitive + Ceil>(n: T) -> Fallible<Self>;
}

impl<QO: ToPrimitive + NumCast + CheckContinuous + Ceil> DistanceCast for QO {
    fn cast<QI: ToPrimitive + Ceil>(v: QI) -> Fallible<QO> {
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
