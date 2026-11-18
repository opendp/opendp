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

/// The same Rust entry point is called by the native mechanism and by its
/// Verus-checked `BoolRR::invoke` method. The distribution of the returned
/// word, not a privacy guarantee, is trusted.
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

/// Apply deterministic postprocessing to one real production entropy call.
/// This shared executable body is the sampling/composition point for Verus.
/// Only the distribution of `sample_uniform_u64` is external; the callback
/// invocation is checked as ordinary Rust code.
#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, verus_spec(result =>
    requires
        forall|word: Result<u64, WordError>|
            #[trigger] call_requires(postprocess, (operator, input, word)),
    ensures
        exists|word: Result<u64, WordError>|
            call_ensures(postprocess, (operator, input, word), result),
))]
pub(crate) fn sample_word_then<Op, X, Y, F>(
    operator: &Op,
    input: X,
    postprocess: F,
) -> Result<Y, WordError>
where
    F: Fn(&Op, X, Result<u64, WordError>) -> Result<Y, WordError>,
{
    let entropy = sample_uniform_u64();
    postprocess(operator, input, entropy)
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
    spec_fn(&X, Result<u64, WordError>) -> Result<Y, WordError>;

/// The one-word weakest preexpectation, using the *same* uniform entropy
/// primitive as the production code. No privacy claim is trusted here.
pub open spec fn one_word_expectation<X, Y>(
    input: &X,
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
