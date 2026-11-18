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

// The mathematical expectation functional of the *production* entropy call.
// Its law is the sole probabilistic primitive assumed by this proof stack.
// This is not an independent or caller-selected probabilistic model.
#[cfg(verus_only)]
verus! {
pub uninterp spec fn uniform_word_expectation(
    payoff: spec_fn(Result<u64, WordError>) -> real,
) -> real;

/// Trusted distributional law of sample_uniform_u64, including the
/// input-independent error path. This statement is about entropy, not DP.
/// The executable RNG call itself has no native Verus probabilistic semantics.
#[verifier::external_body]
pub proof fn lemma_uniform_word_expectation_rule()
    ensures crate::bernoulli_proof::uniform_word_expectation_rule(
        uniform_word_expectation),
{
}
}
