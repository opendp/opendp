use crate::error::Fallible;

pub(crate) mod bigint;
pub(crate) mod float;
pub(crate) mod integer;
use dashu::{integer::IBig, rational::RBig};
use float::get_min_k;

use super::ZExpFamily;

pub trait Nature: Sized {
    /// The random variable of given P-norm specific to the type of self.
    type RV<const P: usize>;
    /// For any parameterization,
    /// returns an equivalent random variable corresponding to the nature of `Self`,
    /// or error.
    fn new_distribution<const P: usize>(
        scale: f64,
        k: Option<i32>,
        radius: Option<Self>,
    ) -> Fallible<Self::RV<P>>;
}

impl Nature for IBig {
    type RV<const P: usize> = ZExpFamily<P>;
    fn new_distribution<const P: usize>(
        scale: f64,
        k: Option<i32>,
        radius: Option<IBig>,
    ) -> Fallible<Self::RV<P>> {
        if k.unwrap_or(0) != 0 {
            return fallible!(MakeMeasurement, "k is only valid for domains over floats");
        }
        if let Some(ref r) = radius {
            if r <= &IBig::ZERO {
                return fallible!(MakeMeasurement, "radius must be positive");
            }
        }
        Ok(ZExpFamily::<P> {
            scale: RBig::try_from(scale)?,
            radius: radius.map(|r| {
                let (_, unsigned_radius) = r.into_parts();
                unsigned_radius
            }),
        })
    }
}

macro_rules! impl_Nature_float {
    ($($T:ty)+) => ($(impl Nature for $T {
        type RV<const P: usize> = float::FloatExpFamily<P, $T>;
        fn new_distribution<const P: usize>(scale: f64, k: Option<i32>, radius: Option<$T>) -> Fallible<Self::RV<P>> {
            Ok(float::FloatExpFamily::<P,$T> {
                scale,
                k: k.unwrap_or_else(get_min_k::<$T>),
                radius
            })
        }
    })+)
}

macro_rules! impl_Nature_int {
    ($($T:ty)+) => ($(impl Nature for $T {
        type RV<const P: usize> = integer::IntExpFamily<P, $T>;
        fn new_distribution<const P: usize>(
            scale: f64,
            k: Option<i32>,
            radius: Option<$T>
        ) -> Fallible<Self::RV<P>> {
            if k.unwrap_or(0) != 0 {
                return fallible!(MakeMeasurement, "k is only valid for domains over floats");
            }
            Ok(integer::IntExpFamily::<P,$T> {
                scale,
                radius
            })
        }
    })+)
}

// these implementations are not proof dependencies,
// and thus do not need privacy proofs
impl_Nature_float!(f32 f64);
impl_Nature_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
