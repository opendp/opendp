use super::*;
#[cfg(feature = "contrib")]
use crate::{combinators::CompositionMeasure, measures::MultiDP};
use crate::{
    error::ErrorVariant,
    measures::{PrivacyProfile, curves::test::batch},
};

fn rdp_epsilon(curve: &PrivacyGuarantee, alpha: f64) -> Fallible<f64> {
    super::super::renyidp::evaluate_renyi(curve.renyi.as_ref().unwrap().curve.as_ref(), &[alpha])
        .map(|values| values[0])
}

#[test]
fn test_composed_rdp_vector_queries() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_renyiDP(batch(|alpha| Ok(0.5 * alpha)), 0.0)?;
    let second = PrivacyGuarantee::new().with_renyiDP(batch(|_| Ok(0.5)), 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![first, second])?;
    let curve = composed.renyi.as_ref().unwrap().curve.as_ref();
    assert_eq!(
        super::super::renyidp::evaluate_renyi(curve, &[2.0, 2.0, 4.0])?,
        vec![1.5, 1.5, 2.5]
    );
    assert!(super::super::renyidp::evaluate_renyi(curve, &[])?.is_empty());
    let failing = PrivacyGuarantee::new()
        .with_renyiDP(|_| fallible!(NumericError, "native RDP failed"), 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![failing])?;
    let error = rdp_epsilon(&composed, 2.0).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::NumericError);
    assert_eq!(error.message.as_deref(), Some("native RDP failed"));
    Ok(())
}

#[test]
fn test_composition_retains_supported_approximate_zcdp() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_zCDP(0.1, 0.1)?;
    let second = PrivacyGuarantee::new().with_zCDP(0.2, 0.2)?;
    let composed = PrivacyGuarantee::compose(vec![first, second])?;

    assert!(composed.renyi.is_none());
    let zcdp = composed.zcdp.unwrap();
    assert!(zcdp.rho >= 0.3);
    assert!((0.3..0.4).contains(&zcdp.source_delta));
    Ok(())
}

#[test]
fn test_zcdp_and_rdp_compose_via_rdp() -> Fallible<()> {
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?;
    let rdp = PrivacyGuarantee::new().with_renyiDP(batch(|_| Ok(0.1)), 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![zcdp, rdp])?;

    assert!(composed.renyi.is_some());
    assert!(composed.zcdp.is_none());
    assert!(rdp_epsilon(&composed, 2.0)? >= 0.5);
    Ok(())
}

#[cfg(feature = "contrib")]
#[test]
fn test_sequential_composition_uses_guarantee_composition() -> Fallible<()> {
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?;
    let rdp = PrivacyGuarantee::new().with_renyiDP(batch(|_| Ok(0.1)), 0.0)?;
    let composed = MultiDP::default().compose(vec![zcdp, rdp])?;

    assert!(composed.renyi.is_some());
    assert!(composed.zcdp.is_none());
    Ok(())
}

#[test]
fn test_rdp_uses_tighter_native_or_zcdp_bound_pointwise() -> Fallible<()> {
    let curve = PrivacyGuarantee::new()
        .with_renyiDP(
            batch(|alpha| Ok(if alpha < 3.0 { alpha * 2.0 } else { 1.0 })),
            0.0,
        )?
        .with_zCDP(1.0, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![curve])?;

    assert_eq!(rdp_epsilon(&composed, 2.0)?, 2.0);
    assert_eq!(rdp_epsilon(&composed, 4.0)?, 1.0);
    Ok(())
}

#[test]
fn test_approximate_rdp_does_not_intersect_exact_zcdp() -> Fallible<()> {
    let curve = PrivacyGuarantee::new()
        .with_renyiDP(batch(|_| Ok(0.5)), 0.1)?
        .with_zCDP(10.0, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![curve])?;

    // The approximate native RDP fact is not pointwise-minimized with exact
    // zCDP. The independent exact zCDP path is used instead.
    assert_eq!(rdp_epsilon(&composed, 2.0)?, 20.0);
    assert_eq!(composed.renyi.unwrap().source_delta, 0.0);
    Ok(())
}

#[test]
fn test_rdp_zcdp_and_zcdp_retain_both() -> Fallible<()> {
    let both = PrivacyGuarantee::new()
        .with_renyiDP(batch(|_| Ok(0.5)), 0.0)?
        .with_zCDP(0.1, 0.0)?;
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![both, zcdp])?;

    assert!(composed.renyi.is_some());
    assert!(composed.zcdp.is_some());
    assert!(rdp_epsilon(&composed, 2.0)? >= 0.6);
    assert!(composed.zcdp.unwrap().rho >= 0.3);
    Ok(())
}

#[test]
fn test_profile_only_composition_has_no_supported_path() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_profile(PrivacyProfile::new(batch(|_| Ok(0.1))));
    let second = PrivacyGuarantee::new().with_profile(PrivacyProfile::new(batch(|_| Ok(0.2))));
    let error = PrivacyGuarantee::compose(vec![first, second]).unwrap_err();
    assert_eq!(
        error.message.as_deref(),
        Some("PrivacyGuarantee composition has no supported common representation")
    );
    Ok(())
}

#[test]
fn test_exact_certificate_composition() -> Fallible<()> {
    let pure = PrivacyGuarantee::compose(vec![
        PrivacyGuarantee::new().with_approx_dp(0.5, 0.0)?,
        PrivacyGuarantee::new().with_approx_dp(0.25, 0.0)?,
    ])?;
    assert_eq!(pure.approx_dp(), Some((0.75, 0.0)));
    assert!(pure.profile.is_none());
    assert_eq!(pure.epsilon(0.0)?, 0.75);

    let approximate = PrivacyGuarantee::compose(vec![
        PrivacyGuarantee::new().with_approx_dp(0.1, 0.2)?,
        PrivacyGuarantee::new().with_approx_dp(0.2, 0.4)?,
    ])?;
    let (epsilon, delta) = approximate.approx_dp().unwrap();
    // Exact rational sums must be enclosed by the reported upper bounds.
    assert!(
        dashu::rational::RBig::try_from(epsilon)?
            >= dashu::rational::RBig::try_from(0.1)? + dashu::rational::RBig::try_from(0.2)?
    );
    assert!(
        dashu::rational::RBig::try_from(delta)?
            >= dashu::rational::RBig::try_from(0.2)? + dashu::rational::RBig::try_from(0.4)?
    );
    assert_eq!(epsilon, 0.1_f64 + 0.2);
    assert_eq!(delta, 0.2_f64 + 0.4);

    let capped = PrivacyGuarantee::compose(vec![
        PrivacyGuarantee::new().with_approx_dp(0.5, 0.75)?,
        PrivacyGuarantee::new().with_approx_dp(0.25, 0.5)?,
    ])?;
    assert_eq!(capped.approx_dp(), Some((0.75, 1.0)));
    let tiny = f64::from_bits(1);
    let subnormal = PrivacyGuarantee::new().with_approx_dp(tiny, tiny)?;
    let composed = PrivacyGuarantee::compose(vec![subnormal.clone(), subnormal])?;
    assert_eq!(composed.approx_dp(), Some((tiny + tiny, tiny + tiny)));
    Ok(())
}

#[test]
fn test_certificate_path_is_independent_of_profiles_and_zcdp() -> Fallible<()> {
    let first = PrivacyGuarantee::new()
        .with_approx_dp(0.5, 0.1)?
        .with_profile(PrivacyProfile::new(|_| {
            fallible!(FailedFunction, "profile must not be composed")
        }))
        .with_zCDP(0.1, 0.0)?;
    let second = PrivacyGuarantee::new()
        .with_approx_dp(0.25, 0.2)?
        .with_zCDP(0.2, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![first.clone(), second])?;
    assert!(composed.approx_dp.is_some());
    assert!(composed.profile.is_none());
    assert!((0.3..0.31).contains(&composed.zcdp.unwrap().rho));

    let mixed =
        PrivacyGuarantee::compose(vec![first, PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?])?;
    assert!(mixed.approx_dp.is_none());
    assert!(mixed.zcdp.is_some());
    assert!(
        PrivacyGuarantee::compose(vec![
            PrivacyGuarantee::new().with_approx_dp(0.5, 0.0)?,
            PrivacyGuarantee::new().with_profile(PrivacyProfile::new(batch(|_| Ok(0.1)))),
        ])
        .is_err()
    );
    Ok(())
}

#[test]
fn test_certificate_overflow_is_not_silently_dropped() -> Fallible<()> {
    let large = PrivacyGuarantee::new().with_approx_dp(f64::MAX, 0.0)?;
    let unchanged = PrivacyGuarantee::compose(vec![
        large.clone(),
        PrivacyGuarantee::new().with_approx_dp(0.0, 0.0)?,
    ])?;
    assert_eq!(unchanged.approx_dp(), Some((f64::MAX, 0.0)));
    let error = PrivacyGuarantee::compose(vec![large.clone(), large.clone()]).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Overflow);
    let both = large.with_zCDP(0.1, 0.0)?;
    let error = PrivacyGuarantee::compose(vec![both.clone(), both]).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::Overflow);
    Ok(())
}

#[test]
fn test_nary_composition() -> Fallible<()> {
    let curves = [0.1, 0.2, 0.3, 0.4]
        .into_iter()
        .map(|rho| PrivacyGuarantee::new().with_zCDP(rho, 0.0))
        .collect::<Fallible<Vec<_>>>()?;
    let composed = PrivacyGuarantee::compose(curves)?;
    assert!(composed.zcdp.unwrap().rho >= 1.0);
    Ok(())
}

#[test]
fn test_rdp_optional_callback_error_uses_independent_zcdp_path() -> Fallible<()> {
    let failing = PrivacyGuarantee::new()
        .with_renyiDP(
            |_| fallible!(FailedFunction, "optional RDP path failed"),
            0.0,
        )?
        .with_zCDP(0.1, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![failing])?;
    assert_eq!(rdp_epsilon(&composed, 2.0)?, 0.2);
    Ok(())
}

#[test]
fn test_approximate_zcdp_delta_is_not_embedded_into_rdp() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_zCDP(0.1, 0.01)?;
    let second = PrivacyGuarantee::new().with_zCDP(0.2, 0.02)?;
    let composed = PrivacyGuarantee::compose(vec![first, second])?;

    assert!(composed.renyi.is_none());
    assert!(composed.zcdp.unwrap().source_delta >= 0.03);
    Ok(())
}

#[test]
fn test_approximate_rdp_composition_requires_a_theorem() -> Fallible<()> {
    let first = PrivacyGuarantee::new()
        .with_renyiDP(batch(|_| Ok(0.1)), 0.03)?
        .with_zCDP(0.2, 0.4)?;
    let second = PrivacyGuarantee::new().with_renyiDP(batch(|_| Ok(0.2)), 0.05)?;
    let error = PrivacyGuarantee::compose(vec![first, second]).unwrap_err();
    assert_eq!(
        error.message.as_deref(),
        Some("PrivacyGuarantee composition has no supported common representation")
    );
    Ok(())
}

#[test]
fn test_composition_identity() -> Fallible<()> {
    let identity = PrivacyGuarantee::compose(vec![])?;
    assert!(identity.profile.is_none());
    assert_eq!(identity.approx_dp(), Some((0.0, 0.0)));
    assert!(identity.renyi.is_some());
    assert!(identity.zcdp.is_some());
    #[cfg(feature = "idealized-numerics")]
    assert!(identity.gaussian.is_some());
    assert_eq!(identity.delta(0.0)?, 0.0);
    Ok(())
}

#[cfg(feature = "idealized-numerics")]
#[test]
fn test_gaussian_composition_rejects_overflow() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_gaussianDP(f64::MAX)?;
    let second = PrivacyGuarantee::new().with_gaussianDP(f64::MAX)?;
    assert!(PrivacyGuarantee::compose(vec![first, second]).is_err());
    Ok(())
}
