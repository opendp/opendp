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
        assert((bits as nat) < (numerator as nat) * 2048nat);
    } else {
        assert((bits as nat) >= (numerator as nat) * 2048nat);
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

/// A payoff on the actual output of sample_uniform_u64(). This expresses
/// exactly the only distributional rule needed for Boolean RR.
pub open spec fn prefix_post<E>(
    threshold: nat, below: real, above: real,
    err_post: spec_fn(E) -> real,
) -> spec_fn(Result<u64, E>) -> real {
    |sample: Result<u64, E>| match sample {
        Ok(word) => if (word as nat) < threshold { below } else { above },
        Err(err) => err_post(err),
    }
}

/// The one probabilistic boundary: expectation preservation for the
/// *production* sample_uniform_u64() call. No Verus code proves this
/// contract from fill_bytes yet. The two expectation functionals must be
/// the semantics of that actual call and its observable errors.
///
/// The rule states the distribution of a uniform 64-bit prefix event,
/// including an arbitrary input-independent error payload.
pub open spec fn uniform_word_expectation_rule<E>(
    word_expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    error_expectation: spec_fn(spec_fn(E) -> real) -> real,
    success_probability: real,
) -> bool {
    &&& 0real <= success_probability <= 1real
    &&& error_expectation(|_: E| 1real) == 1real - success_probability
    &&& forall|err_post: spec_fn(E) -> real|
        (forall|err: E| 0real <= #[trigger] err_post(err) <= 1real)
            ==> 0real <= #[trigger] error_expectation(err_post)
    &&& forall|threshold: nat, below: real, above: real, err_post: spec_fn(E) -> real|
        threshold <= 18_446_744_073_709_551_616nat
        && 0real <= below <= 1real
        && 0real <= above <= 1real
        && (forall|err: E| 0real <= #[trigger] err_post(err) <= 1real)
        ==> #[trigger] word_expectation(prefix_post(threshold, below, above, err_post))
            == success_probability * (
                threshold as real / 18_446_744_073_709_551_616real * below
                + (1real - threshold as real / 18_446_744_073_709_551_616real) * above
            ) + error_expectation(err_post)
}

} // verus!
