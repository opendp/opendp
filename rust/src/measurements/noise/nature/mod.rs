use crate::error::Fallible;

pub(crate) mod bigint;
pub(crate) mod float;
pub(crate) mod integer;
use dashu::{integer::IBig, rational::RBig};
use float::get_min_k;

use super::ZExpFamily;

pub trait Nature {
    /// The random variable of given P-norm specific to the type of self.
    type RV<const P: usize>;
    /// For any parameterization,
    /// returns an equivalent random variable corresponding to the nature of `Self`,
    /// or error.
    fn new_distribution<const P: usize>(scale: f64, k: Option<i32>) -> Fallible<Self::RV<P>>;
}

impl Nature for IBig {
    type RV<const P: usize> = ZExpFamily<P>;
    fn new_distribution<const P: usize>(scale: f64, k: Option<i32>) -> Fallible<Self::RV<P>> {
        if k.unwrap_or(0) != 0 {
            return fallible!(MakeMeasurement, "k is only valid for domains over floats");
        }
        Ok(ZExpFamily::<P> {
            scale: RBig::try_from(scale)?,
        })
    }
}

macro_rules! impl_Nature_float {
    ($($T:ty)+) => ($(impl Nature for $T {
        type RV<const P: usize> = float::FloatExpFamily<P>;
        fn new_distribution<const P: usize>(scale: f64, k: Option<i32>) -> Fallible<Self::RV<P>> {
            Ok(float::FloatExpFamily::<P> {
                scale,
                k: k.unwrap_or_else(get_min_k::<$T>),
            })
        }
    })+)
}
macro_rules! impl_Nature_int {
    ($($T:ty)+) => ($(impl Nature for $T {
        type RV<const P: usize> = integer::IntExpFamily<P>;
        fn new_distribution<const P: usize>(scale: f64, k: Option<i32>) -> Fallible<Self::RV<P>> {
            if k.unwrap_or(0) != 0 {
                return fallible!(MakeMeasurement, "k is only valid for domains over floats");
            }
            Ok(integer::IntExpFamily::<P> {
                scale,
            })
        }
    })+)
}

// these implementations are not proof dependencies,
// and thus do not need privacy proofs
impl_Nature_float!(f32 f64);
impl_Nature_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
