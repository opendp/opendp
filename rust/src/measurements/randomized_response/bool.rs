//! Production public Boolean randomized-response constructor.
//! Imported by both OpenDP and the Verus harness.
#[cfg(not(verus_only))]
use opendp_derive::bootstrap;
#[cfg(verus_only)]
use vstd::prelude::*;

#[cfg(verus_only)]
use crate::{atom_domain::AtomDomain, discrete_distance::DiscreteDistance,
    max_divergence::MaxDivergence, Measurement};
#[cfg(not(verus_only))]
use crate::{core::Measurement, domains::AtomDomain, metrics::DiscreteDistance,
    measures::MaxDivergence};
#[cfg(verus_only)]
type Fallible<T> = Result<T, ()>;
#[cfg(not(verus_only))]
use crate::error::*;
#[cfg(verus_only)]
use crate::rr_constructor as constructor;
#[cfg(not(verus_only))]
use super::constructor;

#[cfg_attr(not(verus_only), bootstrap(features("contrib"), arguments(constant_time(default = false)))))]
/// Make a Measurement that implements randomized response on a boolean value.
///
/// # Citations
/// * [Warner65 Randomized Response: A Survey Technique for Eliminating Evasive Answer Bias](https://doi.org/10.1080/01621459.1965.10480775)
///
/// # Runtime
/// Per release, runtime is `O(1)`.
///
/// # Utility
/// The released bit matches the input with probability `prob`,
/// so the expected 0-1 loss is `O(1 - prob)`.
///
/// # Arguments
/// * `prob` - Probability of returning the correct answer. Must be in `[0.5, 1]`
/// * `constant_time` - Both values use one fixed-size uniform-byte draw.
///
/// # Verification
/// Verus checks the operator builder and conditional privacy guarantee.
/// Entropy, numerical primitives, and callback erasure are trusted; see
/// rust/opendp_verus/README.md.
pub fn make_randomized_response_bool(
    prob: f64,
    constant_time: bool,
) -> Fallible<Measurement<AtomDomain<bool>, DiscreteDistance, MaxDivergence, bool>> {
    // This is the same executable path imported by the Verus harness.
    let measurement = constructor::build_bool_rr_measurement(
        prob.to_bits(),
        AtomDomain::default(),
        DiscreteDistance,
        MaxDivergence,
    )?;
    let _ = constant_time; // Both settings use one fixed-size entropy draw.
    let Some(measurement) = measurement else {
        #[cfg(not(verus_only))]
        return fallible!(MakeMeasurement, "probability must be within [0.5, 1]");
        #[cfg(verus_only)]
        return Err(());
    };
    Ok(measurement)
}

