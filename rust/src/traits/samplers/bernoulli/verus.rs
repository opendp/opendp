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

/// The prefix test on a uniform u64 has exactly numerator / 2^53
/// success probability; this is a theorem about the production threshold.
pub proof fn lemma_bernoulli_probability(numerator: u64)
    requires numerator <= 9_007_199_254_740_992u64,
    ensures
        (prefix_threshold(numerator) as real)
            / 18_446_744_073_709_551_616real
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

/// Derived probability that the actual entropy call returns a word.
pub open spec fn success_mass<E>(
    expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
) -> real {
    expectation(|sample: Result<u64, E>| match sample {
        Ok(_) => 1real,
        Err(_) => 0real,
    })
}

/// Derived contribution of an observable error payload.
pub open spec fn error_mass<E>(
    expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    payoff: spec_fn(E) -> real,
) -> real {
    expectation(|sample: Result<u64, E>| match sample {
        Ok(_) => 0real,
        Err(err) => payoff(err),
    })
}

/// The zero payoff on an error is an identically zero event.
pub open spec fn zero_error_post<E>() -> spec_fn(E) -> real {
    |_err: E| 0real
}

/// The one probabilistic boundary: the expectation functional of the
/// production sample_uniform_u64 call. Error mass and success mass are
/// derived from this *same* functional, not independently supplied axioms.
///
/// Conditional on a successful entropy draw, the u64 word is uniform.
/// Errors (including payloads) are input-independent.
pub open spec fn uniform_word_expectation_rule<E>(
    expectation: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
) -> bool {
    &&& 0real <= success_mass(expectation) <= 1real
    &&& error_mass(expectation, |_: E| 1real) == 1real - success_mass(expectation)
    // An expectation must send the impossible event to zero. The old rule
    // did not constrain error_mass(0), admitting non-probability functionals.
    &&& error_mass(expectation, zero_error_post()) == 0real
    &&& forall|err_post: spec_fn(E) -> real|
        (forall|err: E| 0real <= #[trigger] err_post(err) <= 1real)
        ==> 0real <= #[trigger] error_mass(expectation, err_post)
            <= 1real - success_mass(expectation)
    &&& forall|threshold: nat, below: real, above: real, err_post: spec_fn(E) -> real|
        threshold <= 18_446_744_073_709_551_616nat
        && 0real <= below <= 1real
        && 0real <= above <= 1real
        && (forall|err: E| 0real <= #[trigger] err_post(err) <= 1real)
        ==> #[trigger] expectation(prefix_post(threshold, below, above, err_post))
            == success_mass(expectation) * (
                threshold as real / 18_446_744_073_709_551_616real * below
                + (1real - threshold as real / 18_446_744_073_709_551_616real) * above
            ) + error_mass(expectation, err_post)
}

/// The exact mathematical value of any accepted binary64 probability bit
/// pattern. Accepted probabilities lie in [1/2, 1], so the only nontrivial
/// exponent is 1022 and every value has denominator 2^53.
pub open spec fn accepted_binary64_value(bits: u64) -> real {
    if bits == 0x3ff0_0000_0000_0000u64 {
        1real
    } else {
        ((4_503_599_627_370_496nat
            + (bits - 0x3fe0_0000_0000_0000u64) as nat) as real)
            / 9_007_199_254_740_992real
    }
}

/// The *production* bit decoder gives the exact numerator, not a rounded
/// conversion of p to a rational. No rational sampler model is involved.
/// The separately audited IEEE-to-bits operation is the remaining boundary
/// connecting the input f64 to this bit-level statement.
pub proof fn lemma_accepted_binary64_bits(bits: u64)
    requires
        0x3fe0_0000_0000_0000u64 <= bits
            <= 0x3ff0_0000_0000_0000u64,
    ensures
        accepted_binary64_value(bits)
            == dyadic53::numerator_from_bits(bits) as real
                / 9_007_199_254_740_992real,
{
    if bits == 0x3ff0_0000_0000_0000u64 {
    } else {
        assert(bits < 0x3ff0_0000_0000_0000u64);
        assert(bits - 0x3fe0_0000_0000_0000u64
            < 4_503_599_627_370_496u64);
        assert(0x3fe0_0000_0000_0000u64
            % 4_503_599_627_370_496u64 == 0u64) by(compute);
        assert(bits % 4_503_599_627_370_496u64
            == bits - 0x3fe0_0000_0000_0000u64);
    }
}


/// The production checked decoder enforces the domain of the constructor.
/// On success it reconstructs exactly the accepted binary64 probability.
pub proof fn lemma_checked_binary64_probability(bits: u64)
    ensures
        match dyadic53::checked_numerator_from_bits(bits) {
            None => bits < 0x3fe0_0000_0000_0000u64
                || bits > 0x3ff0_0000_0000_0000u64,
            Some(numerator) => {
                &&& 4_503_599_627_370_496u64 <= numerator
                &&& numerator <= 9_007_199_254_740_992u64
                &&& accepted_binary64_value(bits) ==
                    numerator as real / 9_007_199_254_740_992real
            },
        },
{
    match dyadic53::checked_numerator_from_bits(bits) {
        None => {},
        Some(numerator) => {
            lemma_accepted_binary64_bits(bits);
            assert(numerator == dyadic53::numerator_from_bits(bits));
        },
    }
}

} // verus!
