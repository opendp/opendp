use crate::{error::Fallible, traits::Integer};

use super::fill_bytes;

use dashu::{base::BitTest, integer::UBig};
use num::Unsigned;
use opendp_derive::proven;

#[cfg(test)]
mod exact_test;
#[cfg(test)]
mod test;

/// Create a value of type `Self` from a byte array of length `N`.
pub trait FromBytes<const N: usize> {
    /// # Proof Definition
    ///
    /// Returns a native endian value of type `Self`
    /// from its representation as a byte array in native endianness.
    fn from_ne_bytes(bytes: [u8; N]) -> Self;
}

macro_rules! impl_from_bytes {
    ($($ty:ty)+) => ($(impl FromBytes<{size_of::<$ty>()}> for $ty {
        fn from_ne_bytes(bytes: [u8; size_of::<$ty>()]) -> Self {
            <$ty>::from_ne_bytes(bytes)
        }
    })+)
}
impl_from_bytes!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

/// # Proof Definition
/// Return either `Err(e)` if there is insufficient system entropy,
/// or `Some(sample)`, where `sample` is a value of type T filled with uniformly random bits.
pub fn sample_from_uniform_bytes<T: FromBytes<N>, const N: usize>() -> Fallible<T> {
    let mut buffer = [0; N];
    fill_bytes(&mut buffer)?;
    Ok(T::from_ne_bytes(buffer))
}

#[proven]
/// Sample an integer uniformly from `[0, upper)`
///
/// # Proof Definition
/// For positive `upper`, return either `Err(e)` if there is insufficient system
/// entropy, or `Ok(sample)`, uniformly distributed over `[0, upper)`.
/// Return an error if `upper` is zero.
pub fn sample_uniform_uint_below<T: Integer + Unsigned + FromBytes<N>, const N: usize>(
    upper: T,
) -> Fallible<T> {
    sample_uniform_uint_below_with::<T, N>(upper, &mut fill_bytes)
}

pub(super) fn sample_uniform_uint_below_with<
    T: Integer + Unsigned + FromBytes<N>,
    const N: usize,
>(
    upper: T,
    fill: &mut impl FnMut(&mut [u8]) -> Fallible<()>,
) -> Fallible<T> {
    if upper == T::zero() {
        return fallible!(FailedFunction, "uniform upper bound must be positive");
    }
    if upper == T::one() {
        return Ok(T::zero());
    }
    let threshold = T::MAX_FINITE - T::MAX_FINITE % upper;

    Ok(loop {
        // algorithm is only valid when sample is non-negative, which is why T: Unsigned
        let mut bytes = [0u8; N];
        fill(&mut bytes)?;
        let sample = T::from_ne_bytes(bytes);
        if sample < threshold {
            // sample % upper is unbiased for any v < MAX_FINITE - MAX_FINITE % upper, because
            // MAX_FINITE - MAX_FINITE % upper evenly folds into [0, upper), MAX_FINITE // upper times
            break sample % upper;
        }
    })
}

#[proven]
/// Sample an integer uniformly from `[0, upper)`
///
/// # Proof Definition
/// For positive `upper`, return either `Err(e)` if there is insufficient system
/// entropy, or `Ok(sample)`, uniformly distributed over `[0, upper)`.
/// Return an error if `upper` is zero.
pub fn sample_uniform_ubig_below(upper: UBig) -> Fallible<UBig> {
    sample_uniform_ubig_below_with(upper, &mut fill_bytes)
}

pub(super) fn sample_uniform_ubig_below_with(
    upper: UBig,
    fill: &mut impl FnMut(&mut [u8]) -> Fallible<()>,
) -> Fallible<UBig> {
    if upper.is_zero() {
        return fallible!(FailedFunction, "uniform upper bound must be positive");
    }
    if upper == UBig::ONE {
        return Ok(UBig::ZERO);
    }
    // Sample ceil(log2(upper)) bits. Mask padding in the first big-endian
    // byte, then reject outside [0, upper). No wide division or remainder.
    // For power-of-two upper bounds, every candidate is accepted.
    let bits = (&upper - 1u8).bit_len();
    let mut buffer = vec![0; bits.div_ceil(8)];
    let high_bits = bits % 8;
    Ok(loop {
        fill(&mut buffer)?;
        if high_bits != 0 {
            buffer[0] &= (1u8 << high_bits) - 1;
        }
        let sample = UBig::from_be_bytes(&buffer);
        if sample < upper {
            break sample;
        }
    })
}
