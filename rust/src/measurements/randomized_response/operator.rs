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
pub struct BoolRR<P: Copy> {
    pub numerator: u64,
    pub privacy_cost: P,
}

impl<P: Copy> BoolRR<P> {
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
    /// The map installed in OpenDP's returned Measurement reads the same
    /// concrete operator state as its function callback.
    #[cfg_attr(verus_only, verus_verify(dual_spec, open))]
    #[cfg_attr(verus_only, verus_spec(result =>
        ensures result == (if distance == 0 { zero } else { self.privacy_cost }),
    ))]
    pub fn map(&self, distance: u32, zero: P) -> P {
        if distance == 0 { zero } else { self.privacy_cost }
    }
}
