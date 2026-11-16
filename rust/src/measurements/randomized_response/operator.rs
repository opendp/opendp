//! Concrete randomized-response Boolean operator, shared by production
//! OpenDP and the Verus harness. Neither file contains a reference copy.
#[cfg(verus_only)]
use vstd::prelude::*;
#[cfg(verus_only)]
use crate::{dyadic53, kernel};
#[cfg(not(verus_only))]
use crate::traits::samplers::dyadic53;
#[cfg(not(verus_only))]
use super::kernel;

#[derive(Clone, Copy)]
pub struct BoolRR {
    pub numerator: u64,
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
        let honest = dyadic53::from_word(self.numerator, entropy);
        kernel::release_result(truth, honest)
    }
}

/// The same privacy-map selection as is installed in the returned Measurement.
/// The privacy cost itself is calculated once from the public numerator.
#[cfg_attr(verus_only, verus_verify(dual_spec, open))]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures result == (if distance == 0 { zero } else { cost }),
))]
pub fn privacy_map<T: Copy>(distance: u32, zero: T, cost: T) -> T {
    if distance == 0 { zero } else { cost }
}
