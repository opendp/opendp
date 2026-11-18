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
/// This rule is an external probabilistic axiom whose soundness is conditional
/// on the error-credit semantics; ordinary Verus alone does not establish
/// its correspondence with the host RNG.
#[cfg_attr(verus_only, verus_verify(external_body))]
#[cfg_attr(verus_only, verus_spec(result =>
    with
        Tracked(start): Tracked<crate::one_word_credit::ErrorCredit>,
        Ghost(word_cost): Ghost<spec_fn(Result<u64, WordError>) -> real>
        -> end: Tracked<crate::one_word_credit::ErrorCredit>
    requires
        start@ =~= crate::one_word_credit::CreditCarrier::Value {
            amount: uniform_word_expectation(word_cost)
        },
        forall|word: Result<u64, WordError>|
            0real <= #[trigger] word_cost(word) < 1real,
    ensures
        end@@ =~= crate::one_word_credit::CreditCarrier::Value {
            amount: word_cost(result)
        },
))]
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

/// A one-draw expectation-preservation proof for the *production* program.
/// The input credit is the primitive word-law expectation of an arbitrary
/// bounded allocation. The verified deterministic callback transports the
/// credit to the observed output. This is the one-word EPT rule; its
/// probabilistic interpretation uses the error-credit soundness argument.
///
/// Importantly, the only transfer of an input credit to a sample-dependent
/// credit occurs at the actual `sample_uniform_u64()` call below. Replacing
/// that call with a constant word cannot satisfy this verified contract.
#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, verus_spec(result =>
    with
        Tracked(start): Tracked<crate::one_word_credit::ErrorCredit>,
        Ghost(word_cost): Ghost<spec_fn(Result<u64, WordError>) -> real>,
        Ghost(output_cost): Ghost<spec_fn(Result<Y, WordError>) -> real>
        -> end: Tracked<crate::one_word_credit::ErrorCredit>
    requires
        start@ =~= crate::one_word_credit::CreditCarrier::Value {
            amount: uniform_word_expectation(word_cost)
        },
        forall|word: Result<u64, WordError>|
            0real <= #[trigger] word_cost(word) < 1real,
        forall|word: Result<u64, WordError>|
            #[trigger] call_requires(postprocess, (operator, input, word)),
        forall|word: Result<u64, WordError>, out: Result<Y, WordError>|
            #[trigger] call_ensures(postprocess, (operator, input, word), out)
                ==> word_cost(word) == output_cost(out),
    ensures
        end@@ =~= crate::one_word_credit::CreditCarrier::Value {
            amount: output_cost(result)
        },
))]
pub(crate) fn sample_word_then<Op, X, Y, F>(
    operator: &Op,
    input: X,
    postprocess: F,
) -> Result<Y, WordError>
where
    X: Copy,
    F: Fn(&Op, X, Result<u64, WordError>) -> Result<Y, WordError>,
{
    #[cfg_attr(verus_only, verus_spec(with
        Tracked(start), Ghost(word_cost) => Tracked(draw_credit)
    ))]
    let entropy = sample_uniform_u64();
    let output = postprocess(operator, input, entropy);
    #[cfg(verus_only)]
    {
        proof! {
            assert(call_ensures(postprocess, (operator, input, entropy), output));
            assert(word_cost(entropy) == output_cost(output));
            assert(draw_credit@ =~= crate::one_word_credit::CreditCarrier::Value {
                amount: output_cost(output)
            });
        }
        #[verus_spec(with |= Tracked(draw_credit))]
        output
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
pub uninterp spec fn uniform_word_expectation(
    payoff: spec_fn(Result<u64, WordError>) -> real,
) -> real;
pub open spec fn word_expectation() ->
    spec_fn(spec_fn(Result<u64, WordError>) -> real) -> real
{
    |payoff: spec_fn(Result<u64, WordError>) -> real|
        uniform_word_expectation(payoff)
}

/// Ghost model of deterministic postprocessing after one entropy result.
/// The executable callback must prove every returned value agrees with it.
pub type OneWordModel<X, Y> =
    spec_fn(X, Result<u64, WordError>) -> Result<Y, WordError>;

/// The one-word weakest preexpectation, using the *same* uniform entropy
/// primitive as the production code. No privacy claim is trusted here.
pub open spec fn one_word_expectation<X, Y>(
    input: X,
    model: OneWordModel<X, Y>,
    payoff: spec_fn(Result<Y, WordError>) -> real,
) -> real {
    word_expectation()(|entropy: Result<u64, WordError>| payoff(model(input, entropy)))
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
