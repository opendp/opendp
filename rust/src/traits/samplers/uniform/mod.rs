use crate::{error::Fallible, traits::Integer};

use super::fill_bytes;

use dashu::{base::BitTest, integer::UBig};
use num::Unsigned;
use opendp_derive::proven;

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
/// For any setting of `upper`,
/// return either `Err(e)` if there is insufficient system entropy,
/// or `Some(sample)`, where `sample` is uniformly distributed over `[0, upper)`.
pub fn sample_uniform_uint_below<T: Integer + Unsigned + FromBytes<N>, const N: usize>(
    upper: T,
) -> Fallible<T> {
    let threshold = T::MAX_FINITE - T::MAX_FINITE % upper;

    Ok(loop {
        // algorithm is only valid when sample is non-negative, which is why T: Unsigned
        let sample = sample_from_uniform_bytes::<T, N>()?;
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
/// For any non-negative setting of `upper`,
/// return either `Err(e)` if there is insufficient system entropy,
/// or `Some(sample)`, where `sample` is uniformly distributed over `[0, upper)`.
pub fn sample_uniform_ubig_below(upper: UBig) -> Fallible<UBig> {
    // ceil(ceil(log_2(upper)) / 8)
    let byte_len = upper.bit_len().div_ceil(8);

    // sample % upper is unbiased for any sample < threshold, because
    // max - max % upper evenly folds into [0, upper), max // upper times
    let max = UBig::from_be_bytes(&vec![u8::MAX; byte_len]);
    let threshold = &max - &max % &upper;

    let mut buffer = vec![0; byte_len];

    Ok(loop {
        fill_bytes(&mut buffer)?;

        let sample = UBig::from_be_bytes(&buffer);
        if sample < threshold {
            break sample % &upper;
        }
    })
}
