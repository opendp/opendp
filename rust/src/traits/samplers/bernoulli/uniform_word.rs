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

/// The actual native entropy primitive. Its *only* probabilistic assumption
/// is the Alerus-style error-credit transfer for an arbitrary bounded
/// allocation on the returned word or error. No postprocessing is trusted.
///
/// The RNG implementation itself carries no probabilistic postcondition;
/// its one trusted expectation transfer is sample_uniform_u64_with_credit.
#[cfg_attr(verus_only, verus_verify(external_body))]
pub(crate) fn sample_uniform_u64() -> Result<u64, WordError> {
    #[cfg(not(verus_only))]
    {
        let mut bytes = [0u8; 8];
        fill_bytes(&mut bytes)?;
        Ok(u64::from_ne_bytes(bytes))
    }
    #[cfg(verus_only)]
    {
        unimplemented!()
    }
}

// This is the single trusted probability-bearing entropy contract.
// It invokes the actual sample_uniform_u64 implementation, adding only an
// erased in-band credit transfer. No other RNG credit axiom is retained.
#[cfg(verus_only)]
#[verus_verify(external_body)]
#[verus_spec(result =>
    requires
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: uniform_word_expectation(word_cost@)
        }),
        forall|word: Result<u64, WordError>|
            0real <= #[trigger] word_cost@(word) < 1real,
    ensures
        result.1@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: word_cost@(result.0)
        }),
)]
pub fn sample_uniform_u64_with_credit(
    credit: Tracked<crate::one_word_credit::ErrorCredit>,
    word_cost: Ghost<WordCost>,
) -> (Result<u64, WordError>, Tracked<crate::one_word_credit::ErrorCredit>) {
    (sample_uniform_u64(), Tracked::assume_new())
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
/// Importantly, the only transfer of an input credit to a sample-dependent
/// credit occurs at the actual `sample_uniform_u64()` call below. Replacing
/// that call with a constant word cannot satisfy this verified contract.
#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        credit@@ =~= (crate::one_word_credit::CreditCarrier::Value {
            amount: uniform_word_expectation(word_cost@)
        }),
        forall|word: Result<u64, WordError>|
            0real <= #[trigger] word_cost@(word) < 1real,
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
    let (entropy, draw_credit) = sample_uniform_u64_with_credit(credit, word_cost);
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

// The mathematical expectation functional of the *production* entropy call.
// Its law is the sole probabilistic primitive assumed by this proof stack.
// This is not an independent or caller-selected probabilistic model.
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
