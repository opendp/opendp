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
        match dyadic53::from_word(self.numerator, entropy) {
            Ok(honest) => Ok(truth ^ !honest),
            Err(err) => Err(err),
        }
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

// The actual native invocation is verified from this same executable body.
// Only the sample_uniform_u64 distribution is outside Verus's scope.
#[cfg(verus_only)]
use crate::uniform_word::{sample_uniform_u64, WordError};
#[cfg(not(verus_only))]
use crate::traits::samplers::sample_uniform_u64;
#[cfg(not(verus_only))]
use crate::error::Error as WordError;

impl BoolRR {
    /// Preserve the deterministic postprocessing of the *actual* entropy
    /// draw. If p=1 and the draw succeeds, the result must equal truth.
    #[cfg_attr(verus_only, verus_verify)]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures self.numerator == 9_007_199_254_740_992u64 && result.is_ok()
            ==> result == Ok(truth),
    ))]
    pub fn invoke(&self, truth: bool) -> Result<bool, WordError> {
        let entropy = sample_uniform_u64();
        self.release(truth, entropy)
    }
}
