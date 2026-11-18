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
use crate::verified_arithmetic::f64_to_bits_exact as bits_exact;
#[cfg(not(verus_only))]
use crate::traits::verified::f64_to_bits_exact as bits_exact;
#[cfg(verus_only)]
use crate::rr_constructor as constructor;
#[cfg(not(verus_only))]
use super::constructor;

#[cfg_attr(not(verus_only), bootstrap(features("contrib"), arguments(constant_time(default = false))))]
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
/// Verus checks this public constructor through Measurement::new_certified.
/// The entropy law, numerical primitives and callback erasure have explicitly
/// documented trust contracts; see rust/opendp_verus/README.md.
#[cfg_attr(verus_only, verus_verify)]
#[cfg_attr(verus_only, verus_spec(result =>
    ensures match result {
        Ok(measurement) => {
            &&& crate::measurement_proof::valid_bool_rr_measurement(&measurement)
            &&& measurement.input_domain.is_unbounded_nonnull()
        },
        Err(_) => true,
    },
))]
pub fn make_randomized_response_bool(
    prob: f64,
    constant_time: bool,
) -> Fallible<Measurement<AtomDomain<bool>, DiscreteDistance, MaxDivergence, bool>> {
    // This is the same executable path imported by the Verus harness.
    let measurement = constructor::build_bool_rr_measurement(
        bits_exact(prob),
        AtomDomain::<bool>::new_non_nan(),
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


#[cfg(all(test, not(verus_only)))]
mod tests {
    use super::*;
    use crate::core::Domain;

    #[test]
    fn public_boolean_rr_uses_expected_privacy_map() {
        let infinite = make_randomized_response_bool(1.0, false).unwrap();
        assert_eq!(infinite.map(&0).unwrap(), 0.0);
        assert_eq!(infinite.map(&1).unwrap(), f64::INFINITY);

        let finite = make_randomized_response_bool(0.75, true).unwrap();
        assert_eq!(finite.map(&0).unwrap(), 0.0);
        assert!(finite.map(&1).unwrap().is_finite());

        assert!(make_randomized_response_bool(0.49, false).is_err());
    }

    #[test]
    fn boolean_non_null_domain_matches_default() {
        let domain = AtomDomain::<bool>::new_non_nan();
        assert_eq!(domain, AtomDomain::<bool>::default());
        assert!(domain.member(&false).unwrap());
        assert!(domain.member(&true).unwrap());
    }
}
