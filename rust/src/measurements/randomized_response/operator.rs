//! Concrete randomized-response Boolean operator, shared by production
//! OpenDP and the Verus harness. Neither file contains a reference copy.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::dyadic53;
#[cfg(not(verus_only))]
use crate::traits::samplers::dyadic53;

#[cfg_attr(verus_only, verus_verify)]
#[derive(Clone, Copy)]
pub struct BoolRR {
    pub numerator: u64,
    pub privacy_cost: f64,
}

impl BoolRR {
    /// Apply one fallible entropy word to the private Boolean. The actual
    /// constructor calls this method, with entropy supplied by fill_bytes.
    #[cfg_attr(verus_only, verus_verify(dual_spec, open))]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures result == match entropy {
            Ok(word) => Ok(truth ^ !((word / 2048u64) < self.numerator)),
            Err(err) => Err(err),
        },
    ))]
    pub fn release<E>(&self, truth: bool, entropy: Result<u64, E>) -> Result<bool, E> {
        match entropy {
            Ok(word) => Ok(truth ^ !dyadic53::success(self.numerator, word)),
            Err(err) => Err(err),
        }
    }
    /// Complete deterministic postprocessing contract, with the signature
    /// needed for a generic one-word sampler. This body is executable in
    /// production and verified against the existing release implementation.
    #[cfg_attr(verus_only, verus_verify)]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures result == self.release(*truth, entropy),
    ))]
    pub fn release_callback(
        &self,
        truth: &bool,
        entropy: Result<u64, WordError>,
    ) -> Result<bool, WordError> {
        self.release(*truth, entropy)
    }

    /// The map installed in OpenDP's returned Measurement reads the same
    /// concrete operator state as its function callback.
    #[cfg_attr(verus_only, verus_verify(dual_spec, open))]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures result == (if distance == 0 { zero } else { self.privacy_cost }),
    ))]
    pub fn map(&self, distance: u32, zero: f64) -> f64 {
        if distance == 0 { zero } else { self.privacy_cost }
    }

}

// The native RR closure calls sample_word_then with release_callback.
// Keep one executable sampling path; do not certify a duplicate sampler.
#[cfg(verus_only)]
use crate::uniform_word::WordError;
#[cfg(not(verus_only))]
use crate::error::Error as WordError;

#[cfg(verus_only)]
verus! {
/// Instantiate the one-word EPT preconditions for the actual Boolean RR
/// postprocessor. The arbitrary output allocation is pulled back along the
/// verified executable release callback; there is no sampler model substituted
/// for the production callback.
pub proof fn lemma_release_callback_preserves_cost(
    operator: BoolRR,
    truth: &bool,
    output_cost: spec_fn(Result<bool, WordError>) -> real,
)
    requires
        forall|out: Result<bool, WordError>|
            0real <= #[trigger] output_cost(out),
    ensures
        forall|word: Result<u64, WordError>|
            0real <= #[trigger] output_cost(operator.release(*truth, word)),
        forall|word: Result<u64, WordError>|
            #[trigger] call_requires(BoolRR::release_callback, (&operator, truth, word)),
        forall|word: Result<u64, WordError>, out: Result<bool, WordError>|
            #[trigger] call_ensures(
                BoolRR::release_callback, (&operator, truth, word), out)
            ==> output_cost(operator.release(*truth, word)) == output_cost(out),
{
    assert forall|word: Result<u64, WordError>|
        #[trigger] call_requires(BoolRR::release_callback, (&operator, truth, word))
    by {}
    assert forall|word: Result<u64, WordError>, out: Result<bool, WordError>|
        #[trigger] call_ensures(
            BoolRR::release_callback, (&operator, truth, word), out)
        implies output_cost(operator.release(*truth, word)) == output_cost(out)
    by {
        assert(out == operator.release(*truth, word));
    }
}
} // verus!
