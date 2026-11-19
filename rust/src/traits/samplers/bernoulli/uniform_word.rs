//! One production entropy draw for Boolean randomized response.
//!
//! Verus treats only the RNG as external. Its expectation law, including
//! input-independent errors, is the explicit probabilistic trust boundary.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(not(verus_only))]
use super::super::fill_bytes;

#[cfg(verus_only)]
pub type WordError = ();
#[cfg(not(verus_only))]
pub type WordError = crate::error::Error;

// The native body is the actual entropy draw. This single entry point
// carries the trusted in-band credit-transfer contract under Verus.
#[cfg(verus_only)]
type CreditWordResult = (
    Result<u64, WordError>, Tracked<crate::one_word_credit::ErrorCredit>,
);
#[cfg(not(verus_only))]
type CreditWordResult = Result<u64, WordError>;

#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: uniform_word_expectation(word_cost@)
        }),
        forall|word: Result<u64, WordError>|
            0real <= #[trigger] word_cost@(word),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: word_cost@(result.0)
        }),
))]
pub(crate) fn sample_uniform_u64(
    #[cfg(verus_only)] credit: Tracked<crate::one_word_credit::ErrorCredit>,
    #[cfg(verus_only)] word_cost: Ghost<WordCost>,
) -> CreditWordResult {
    #[cfg(verus_only)]
    {
        (unimplemented!(), Tracked::assume_new())
    }
    #[cfg(not(verus_only))]
    {
        let mut bytes = [0u8; 8];
        fill_bytes(&mut bytes)?;
        Ok(u64::from_ne_bytes(bytes))
    }
}

#[cfg(verus_only)]
type SampleWordThenResult<Y> = (
    Result<Y, WordError>, Tracked<crate::one_word_credit::ErrorCredit>,
);
#[cfg(not(verus_only))]
type SampleWordThenResult<Y> = Result<Y, WordError>;

/// A one-draw expectation-preservation proof for the *production* program.
/// The in-band input credit is the primitive word-law expectation of an
/// arbitrary bounded allocation. Verified deterministic postprocessing
/// transports that credit to the observed output. This is the one-word EPT
/// rule; its interpretation uses the error-credit soundness argument.
///
/// The only transfer of an input credit to a sample-dependent credit is
/// the trusted call to sample_uniform_u64 below. The verified
/// postprocessor cannot discard the sample and return an unrelated value.
#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: uniform_word_expectation(word_cost@)
        }),
        forall|word: Result<u64, WordError>|
            0real <= #[trigger] word_cost@(word),
        forall|word: Result<u64, WordError>|
            #[trigger] call_requires(postprocess, (operator, input, word)),
        forall|word: Result<u64, WordError>, out: Result<Y, WordError>|
            #[trigger] call_ensures(postprocess, (operator, input, word), out)
                ==> word_cost@(word) == output_cost@(out),
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: output_cost@(result.0)
        }),
))]
pub(crate) fn sample_word_then<Op, X, Y, F>(
    operator: &Op,
    input: X,
    postprocess: F,
    #[cfg(verus_only)] credit: Tracked<crate::one_word_credit::ErrorCredit>,
    #[cfg(verus_only)] word_cost: Ghost<WordCost>,
    #[cfg(verus_only)] output_cost: Ghost<OutputCost<Y>>,
) -> SampleWordThenResult<Y>
where
    X: Copy,
    F: Fn(&Op, X, Result<u64, WordError>) -> Result<Y, WordError>,
{
    #[cfg(verus_only)]
    let (entropy, draw_credit) = sample_uniform_u64(credit, word_cost);
    #[cfg(not(verus_only))]
    let entropy = sample_uniform_u64();
    let output = postprocess(operator, input, entropy);
    #[cfg(verus_only)]
    {
        proof! {
            assert(call_ensures(postprocess, (operator, input, entropy), output));
            assert(word_cost@(entropy) == output_cost@(output));
            assert(draw_credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
                amount: output_cost@(output)
            }));
        }
        (output, draw_credit)
    }
    #[cfg(not(verus_only))]
    {
        output
    }
}

// Mathematical expectation of the actual entropy word. Its distribution
// (including input-independent errors) is separately constrained below.
// This is a fixed RNG model, not caller-selected callback semantics.
#[cfg(verus_only)]
verus! {
/// Type aliases keep higher-order cost types outside attribute signatures.
pub type WordCost = spec_fn(Result<u64, WordError>) -> real;
pub type OutputCost<Y> = spec_fn(Result<Y, WordError>) -> real;

pub uninterp spec fn uniform_word_expectation(
    payoff: spec_fn(Result<u64, WordError>) -> real,
) -> real;
pub open spec fn word_expectation() ->
    spec_fn(spec_fn(Result<u64, WordError>) -> real) -> real
{
    |payoff: spec_fn(Result<u64, WordError>) -> real|
        uniform_word_expectation(payoff)
}

/// Trusted distributional law of sample_uniform_u64, including the
/// input-independent error path. This statement is about entropy, not DP.
/// The executable RNG call itself has no native Verus probabilistic semantics.
#[verifier::external_body]
pub proof fn lemma_uniform_word_expectation_rule()
    ensures crate::bernoulli_proof::uniform_word_expectation_rule(
        word_expectation()),
{
}
}
