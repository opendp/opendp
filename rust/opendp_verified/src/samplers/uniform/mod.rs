use dashu::{base::BitTest, integer::UBig};

use crate::{error::Fallible, samplers::fill_bytes};

/// Sample a fixed number of uniformly random bytes.
///
/// # Proof Definition
/// Return either `Err(e)` if there is insufficient entropy,
/// or `Ok(buffer)`, where `buffer` is uniformly distributed over byte arrays of
/// length `N`.
fn sample_from_uniform_bytes<const N: usize>() -> Fallible<[u8; N]> {
    let mut buffer = [0_u8; N];
    fill_bytes(&mut buffer)?;
    Ok(buffer)
}

/// Sample a `usize` uniformly from `[0, upper)`.
///
/// # Proof Definition
/// For any positive setting of `upper`,
/// return either `Err(e)` if there is insufficient entropy,
/// or `Ok(sample)`, where `sample` is uniformly distributed over `[0, upper)`.
pub fn sample_uniform_usize_below(upper: usize) -> Fallible<usize> {
    let threshold = usize::MAX - usize::MAX % upper;

    Ok(loop {
        let sample = usize::from_ne_bytes(sample_from_uniform_bytes::<{ size_of::<usize>() }>()?);
        if sample < threshold {
            break sample % upper;
        }
    })
}

/// Sample a `UBig` uniformly from `[0, upper)`.
///
/// # Proof Definition
/// For any positive setting of `upper`,
/// return either `Err(e)` if there is insufficient entropy,
/// or `Ok(sample)`, where `sample` is uniformly distributed over `[0, upper)`.
pub fn sample_uniform_ubig_below(upper: UBig) -> Fallible<UBig> {
    // ceil(ceil(log_2(upper)) / 8)
    let byte_len = upper.bit_len().div_ceil(8);

    // sample % upper is unbiased for any sample < threshold, because
    // threshold = 2^(8 * byte_len) - (2^(8 * byte_len) % upper)
    // evenly folds into [0, upper), threshold / upper times
    let range = UBig::ONE << (byte_len * 8);
    let threshold = &range - &range % &upper;

    let mut buffer = vec![0; byte_len];

    Ok(loop {
        fill_bytes(&mut buffer)?;

        let sample = UBig::from_be_bytes(&buffer);
        if sample < threshold {
            break sample % &upper;
        }
    })
}
