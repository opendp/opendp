//! Uniform 64-bit entropy reduces exact Bernoulli to a prefix event.
//! This module proves the deterministic reduction for the *production*
//! dyadic53::success function. It does not axiomatize the entropy provider.
use vstd::prelude::*;
use crate::dyadic53;

verus! {

pub open spec fn prefix_threshold(numerator: u64) -> nat {
    (numerator as nat) * 2048nat
}

/// For 0 <= n <= 2^53, the executable Bernoulli decision is exactly the
/// uniform-u64 prefix event {bits < 2^11 * n}. The raw-word RNG contract
/// can therefore be stated only as the CDF of a uniform 64-bit integer.
pub proof fn lemma_success_is_prefix(numerator: u64, bits: u64)
    requires numerator <= 9_007_199_254_740_992u64,
    ensures
        dyadic53::success(numerator, bits)
            == ((bits as nat) < prefix_threshold(numerator)),
{
    let quotient = bits as nat / 2048nat;
    assert(dyadic53::success(numerator, bits) == (quotient < numerator as nat));
    if quotient < numerator as nat {
        assert(bits as nat < (numerator as nat) * 2048nat);
    } else {
        assert(bits as nat >= (numerator as nat) * 2048nat);
    }
}

/// A uniform 64-bit source has the following exact CDF for 0 <= t <= 2^64.
/// This is a mathematical reference law; the correctness of fill_bytes is
/// the one probabilistic primitive still to justify.
pub open spec fn uniform64_cdf(t: nat) -> real {
    (t as real) / 18_446_744_073_709_551_616real
}

/// The success event is a prefix of the 64-bit uniform range.
pub open spec fn dyadic53_mass(numerator: u64) -> real {
    uniform64_cdf(prefix_threshold(numerator))
}

pub proof fn lemma_bernoulli_probability(numerator: u64)
    requires numerator <= 9_007_199_254_740_992u64,
    ensures
        dyadic53_mass(numerator)
            == (numerator as real) / 9_007_199_254_740_992real,
{
    assert((numerator as real * 2048real) / 18_446_744_073_709_551_616real
        == (numerator as real) / 9_007_199_254_740_992real)
        by(nonlinear_arith);
}

} // verus!
