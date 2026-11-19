//! A fixed-width, exact Bernoulli decision for p in [0.5, 1.0] in binary64.
//! These functions are executed by OpenDP and imported verbatim by Verus.
#[cfg(verus_only)]
use vstd::prelude::*;

/// Under an accepted IEEE-754 binary64 probability p, returns exactly
/// p * 2^53 as an integer. p=1 is the special exponent case.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures
        result as int == if bits == 0x3ff0_0000_0000_0000u64 {
            9_007_199_254_740_992int
        } else {
            4_503_599_627_370_496int + (bits % 4_503_599_627_370_496u64) as int
        },
        result >= 4_503_599_627_370_496u64,
        result <= 9_007_199_254_740_992u64,
))]
pub fn numerator_from_bits(bits: u64) -> u64 {
    if bits == 0x3ff0_0000_0000_0000 {
        9_007_199_254_740_992u64
    } else {
        4_503_599_627_370_496u64 + (bits % 4_503_599_627_370_496u64)
    }
}


/// Validate the exact binary64 encoding before decoding the Bernoulli
/// numerator. The positive f64 interval [0.5, 1] is precisely this
/// contiguous interval of IEEE-754 bit patterns; NaNs, negatives, and
/// out-of-range inputs are rejected without floating-point comparisons.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures result == if 0x3fe0_0000_0000_0000u64 <= bits
        && bits <= 0x3ff0_0000_0000_0000u64 {
        Some(numerator_from_bits(bits))
    } else {
        None
    },
))]
pub fn checked_numerator_from_bits(bits: u64) -> Option<u64> {
    if bits < 0x3fe0_0000_0000_0000u64 || bits > 0x3ff0_0000_0000_0000u64 {
        None
    } else {
        Some(numerator_from_bits(bits))
    }
}

/// Turn uniform 64 bits into Bernoulli(numerator/2^53):
/// truncating the lowest 11 bits produces a uniform 53-bit integer.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures result == ((bits / 2048u64) < numerator),
))]
pub fn success(numerator: u64, bits: u64) -> bool {
    (bits / 2048u64) < numerator
}

/// Return the exact likelihood-ratio numerator and denominator.
/// The only exceptional case is p = 1, for which epsilon is infinite.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        4_503_599_627_370_496u64 <= numerator,
        numerator <= 9_007_199_254_740_992u64,
    ensures
        match result {
            None => numerator == 9_007_199_254_740_992u64,
            Some((n, complement)) =>
                n == numerator
                && complement == 9_007_199_254_740_992u64 - numerator
                && complement > 0u64,
        },
))]
pub fn odds_parts(numerator: u64) -> Option<(u64, u64)> {
    if numerator == 9_007_199_254_740_992u64 {
        None
    } else {
        Some((numerator, 9_007_199_254_740_992u64 - numerator))
    }
}
