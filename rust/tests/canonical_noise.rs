#![cfg(feature = "contrib")]

use opendp::{
    domains::AtomDomain, error::Fallible, measurements::make_canonical_noise,
    measures::PrivacyGuarantee, metrics::AbsoluteDistance,
};

// Run with contrib only: the compiler does not require honest-but-curious.
#[test]
fn public_canonical_noise_from_certificate() -> Fallible<()> {
    let measurement = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        1.0,
        PrivacyGuarantee::new().with_approx_dp(0.7, 1e-7)?,
    )?;
    assert_eq!(measurement.map(&1.0)?.delta(0.7)?, 1e-7);
    assert_eq!(measurement.map(&1.0)?.epsilon(1e-7)?, 0.7);
    assert_eq!(measurement.map(&0.0)?.delta(0.0)?, 0.0);
    assert!(measurement.invoke(&0.0)?.is_finite());
    Ok(())
}
