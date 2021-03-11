use num::{NumCast, ToPrimitive};

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
impl_is_not_continuous!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

// include Ceil on QO to avoid requiring as an additional trait bound in all downstream code
pub trait DistanceCast: NumCast + Ceil + CheckContinuous {
    fn cast<T: ToPrimitive + Ceil>(n: T) -> Option<Self>;
}

impl<QO: ToPrimitive + NumCast + CheckContinuous + Ceil> DistanceCast for QO {
    fn cast<QI: ToPrimitive + Ceil>(v: QI) -> Option<QO> {
        // round away from zero when losing precision
        QO::from(if QO::is_continuous() { v } else { v.ceil() })
    }
}
