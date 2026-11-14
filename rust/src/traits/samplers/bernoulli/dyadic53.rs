//! A fixed-width, exact Bernoulli decision for p in [0.5, 1.0] in binary64.
//! These functions are executed by OpenDP and imported verbatim by Verus.
#[cfg(verus_only)]
use vstd::prelude::*;

/// Under an accepted IEEE-754 binary64 probability p, returns exactly
/// p * 2^53 as an integer. p=1 is the special exponent case.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures
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

/// Turn uniform 64 bits into Bernoulli(numerator/2^53):
/// truncating the lowest 11 bits produces a uniform 53-bit integer.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures result == ((bits >> 11) < numerator),
))]
pub fn success(numerator: u64, bits: u64) -> bool {
    (bits >> 11) < numerator
}
