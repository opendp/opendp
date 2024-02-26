use crate::error::Fallible;

pub(crate) mod float;
pub(crate) mod integer;
use float::get_min_k;

pub trait Nature<const P: usize> {
    type RV;
    fn new_distribution(scale: f64, k: Option<i32>) -> Fallible<Self::RV>;
}

macro_rules! impl_Nature_float {
    ($($T:ty)+) => ($(impl<const P: usize> Nature<P> for $T {
        type RV = float::FloatExpFamily<P>;
        fn new_distribution(scale: f64, k: Option<i32>) -> Fallible<Self::RV> {
            Ok(float::FloatExpFamily::<P> {
                scale,
                k: k.unwrap_or_else(get_min_k::<$T>),
            })
        }
    })+)
}
macro_rules! impl_Nature_int {
    ($($T:ty)+) => ($(impl<const P: usize> Nature<P> for $T {
        type RV = integer::IntExpFamily<P>;
        fn new_distribution(scale: f64, k: Option<i32>) -> Fallible<Self::RV> {
            if k.unwrap_or(0) != 0 {
                return fallible!(MakeMeasurement, "k is only valid for domains over floats");
            }
            Ok(integer::IntExpFamily::<P> {
                scale,
            })
        }
    })+)
}

impl_Nature_float!(f32 f64);
impl_Nature_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
