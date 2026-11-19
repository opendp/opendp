use dashu::{integer::Sign, rational::RBig};
use opendp_derive::proven;

use crate::error::Fallible;

use super::{fill_bytes, sample_uniform_ubig_below};

#[cfg(test)]
mod test;

/// Sample from `Bernoulli(p=0.5)`.
///
/// # Proof Definition
/// Return `Err(e)` if there is insufficient system entropy, or
/// `Ok(sample)`, where `sample` is a draw from `Bernoulli(p=0.5)`.
pub fn sample_standard_bernoulli() -> Fallible<bool> {
    let mut buffer = [0u8; 1];
    fill_bytes(&mut buffer)?;
    Ok(buffer[0] & 1 == 1)
}

/// Round a binary64 probability up to a positive multiple of 2^-64.
/// Zero (from an underflowed positive input) is rounded to 2^-64.
/// The returned threshold is inclusive, so 0 represents probability 2^-64.
/// Multiplication by a power of two is exact for binary64 inputs.
pub(crate) fn bernoulli_u64_threshold(prob: f64) -> u64 {
    debug_assert!((0.0..=1.0).contains(&prob));
    if prob == 1.0 {
        u64::MAX
    } else {
        ((prob * 18446744073709551616.0).ceil() as u64).max(1) - 1
    }
}

/// Sample with probability (threshold + 1) / 2^64, using one uniform word.
pub(crate) fn sample_bernoulli_u64(threshold: u64) -> Fallible<bool> {
    if threshold == u64::MAX {
        return Ok(true);
    }
    let mut bytes = [0u8; 8];
    fill_bytes(&mut bytes)?;
    Ok(u64::from_ne_bytes(bytes) <= threshold)
}

#[proven]
/// Sample a single bit with arbitrary probability of success.
///
/// # Arguments
/// * `prob`- The desired probability of success (bit = 1).
///
/// # Return
/// A true boolean with probability "prob".
///
/// # Proof Definition
/// For any setting of the input parameters,
/// where `prob` is within $[0, 1]$,
/// returns `Err(e)` if there is a lack of system entropy,
/// or `Ok(out)` where `out` is `true` with probability `prob`, otherwise `false`.
pub fn sample_bernoulli_rational(prob: RBig) -> Fallible<bool> {
    let (numer, denom) = prob.into_parts();
    let (Sign::Positive, numer) = numer.into_parts() else {
        return fallible!(FailedFunction, "numerator must not be negative");
    };
    if numer > denom {
        return fallible!(FailedFunction, "prob must not be greater than one");
    }
    sample_uniform_ubig_below(denom).map(|s| numer > s)
}
