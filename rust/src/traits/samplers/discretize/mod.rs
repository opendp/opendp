use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;
use std::convert::TryFrom;

use crate::error::Fallible;
use crate::traits::samplers::sample_discrete_laplace;

use super::sample_discrete_gaussian;

#[allow(non_snake_case)]
/// Sample from the discrete laplace distribution on $\mathbb{Z} \cdot 2^k$.
///
/// Implemented for floating-point types f32 and f64.
///
/// k can be chosen to be very negative,
/// to get an arbitrarily fine approximation to continuous laplacian noise.
///
/// # Proof Definition
/// For any setting of the input arguments, return either
/// `Err(e)` if there is insufficient system entropy, or
/// `Ok(sample)`, where `sample` is distributed according to a modified discrete_laplace(`shift`, `scale`).
///
/// The modifications to the discrete laplace are as follows:
/// - the `shift` is rounded to the nearest multiple of $2^k$
/// - the `sample` is rounded to the nearest value of type `T`.
/// - the noise granularity is in increments of $2^k$.
pub fn sample_discrete_laplace_Z2k<T>(shift: T, scale: T, k: i32) -> Fallible<T>
where
    T: CastInternalRational,
{
    // integerize
    let mut i = find_nearest_multiple_of_2k(shift.into_rational()?, k);

    // sample from the discrete laplace on ℤ*2^k
    i += sample_discrete_laplace(shr(scale.into_rational()?, k))?;

    // postprocess! int -> rational -> T
    Ok(T::from_rational(x_mul_2k(i, k)))
}

#[allow(non_snake_case)]
/// Sample from the discrete gaussian distribution on $\mathbb{Z} \cdot 2^k$.
///
/// Implemented for floating-point types f32 and f64.
///
/// k can be chosen to be very negative,
/// to get an arbitrarily fine approximation to continuous gaussian noise.
///
/// # Proof Definition
/// For any setting of the input arguments, return either
/// `Err(e)` if there is insufficient system entropy, or
/// `Ok(sample)`, where `sample` is distributed according to a modified discrete_gaussian(`shift`, `scale`).
///
/// The modifications to the discrete gaussian are as follows:
/// - the `shift` is rounded to the nearest multiple of $2^k$
/// - the `sample` is rounded to the nearest value of type `T`.
/// - the noise granularity is in increments of $2^k$.
pub fn sample_discrete_gaussian_Z2k<T>(shift: T, scale: T, k: i32) -> Fallible<T>
where
    T: CastInternalRational,
{
    // integerize
    let mut i = find_nearest_multiple_of_2k(shift.into_rational()?, k);

    // sample from the discrete gaussian on ℤ*2^k
    i += sample_discrete_gaussian(shr(scale.into_rational()?, k))?;

    // postprocess! int -> rational -> T
    Ok(T::from_rational(x_mul_2k(i, k)))
}

fn shr(lhs: RBig, rhs: i32) -> RBig {
    let (mut num, mut den) = lhs.into_parts();
    if rhs < 0 {
        num <<= -rhs as usize;
    } else {
        den <<= rhs as usize;
    }

    RBig::from_parts(num, den)
}

/// Find index of nearest multiple of $2^k$ from x.
///
/// # Proof Definition
/// For any setting of input arguments, return the integer $argmin_i |i 2^k - x|$.
fn find_nearest_multiple_of_2k(x: RBig, k: i32) -> IBig {
    // exactly compute shift/2^k and break into fractional parts
    let (sx, sy) = shr(x, k).into_parts();

    // argmin_i |i * 2^k - sx/sy|, the index of nearest multiple of 2^k
    let offset = &sy / UBig::from(2u8) * sx.sign();
    (sx + offset) / sy
}

/// Exactly multiply x by 2^k.
///
/// This is a postprocessing operation.
fn x_mul_2k(x: IBig, k: i32) -> RBig {
    if k > 0 {
        RBig::from(x << k as usize)
    } else {
        RBig::from_parts(x, UBig::ONE << -k as usize)
    }
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

macro_rules! impl_cast_internal_rational_float {
    ($ty:ty, $method:ident) => {
        impl CastInternalRational for $ty {
            fn from_rational(v: RBig) -> Self {
                v.$method().value()
            }
            fn into_rational(self) -> Fallible<RBig> {
                RBig::try_from(self).map_err(|_| err!(FailedFunction, "shift must be finite"))
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

#[cfg(test)]
mod test;
