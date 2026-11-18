//! Concrete randomized-response Boolean operator, shared by production
//! OpenDP and the Verus harness. Neither file contains a reference copy.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::dyadic53;
#[cfg(not(verus_only))]
use crate::traits::samplers::dyadic53;

#[cfg_attr(verus_only, verifier::verify)]
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

// The production invocation is a single entropy call followed by the
// executable, Verus-checked deterministic release method. The RNG itself
// remains the explicit probabilistic trust boundary.
#[cfg(not(verus_only))]
impl BoolRR {
    pub fn invoke(&self, truth: bool) -> crate::error::Fallible<bool> {
        self.release(truth, crate::traits::samplers::sample_uniform_u64())
    }
}
