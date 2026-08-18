use super::*;
use crate::measures::PrivacyProfile;
#[cfg(feature = "contrib")]
use crate::{combinators::CompositionMeasure, measures::MultiDP};

fn rdp_epsilon(curve: &PrivacyGuarantee, alpha: f64) -> Fallible<f64> {
    (curve.renyi.as_ref().unwrap().curve)(alpha)
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
    let rdp = PrivacyGuarantee::new().with_renyiDP_trusted(|_| Ok(0.1), 0.0)?;
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
    let rdp = PrivacyGuarantee::new().with_renyiDP_trusted(|_| Ok(0.1), 0.0)?;
    let composed = MultiDP::default().compose(vec![zcdp, rdp])?;

    assert!(composed.renyi.is_some());
    assert!(composed.zcdp.is_none());
    Ok(())
}

#[test]
fn test_rdp_uses_tighter_native_or_zcdp_bound_pointwise() -> Fallible<()> {
    let curve = PrivacyGuarantee::new()
        .with_renyiDP_trusted(|alpha| Ok(if alpha < 3.0 { alpha * 2.0 } else { 1.0 }), 0.0)?
        .with_zCDP(1.0, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![curve])?;

    assert_eq!(rdp_epsilon(&composed, 2.0)?, 2.0);
    assert_eq!(rdp_epsilon(&composed, 4.0)?, 1.0);
    Ok(())
}

#[test]
fn test_approximate_rdp_does_not_intersect_exact_zcdp() -> Fallible<()> {
    let curve = PrivacyGuarantee::new()
        .with_renyiDP_trusted(|_| Ok(0.5), 0.1)?
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
        .with_renyiDP_trusted(|_| Ok(0.5), 0.0)?
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
    let first = PrivacyGuarantee::from_profile(PrivacyProfile::new(|_| Ok(0.1)));
    let second = PrivacyGuarantee::from_profile(PrivacyProfile::new(|_| Ok(0.2)));
    let error = PrivacyGuarantee::compose(vec![first, second]).unwrap_err();
    assert_eq!(
        error.message.as_deref(),
        Some("PrivacyGuarantee composition has no supported common representation")
    );
    Ok(())
}

#[test]
fn test_point_backed_profile_is_not_composed_as_approximate_dp() -> Fallible<()> {
    let profile = PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(0.1, 0.2)])?;
    assert!(
        PrivacyGuarantee::compose(vec![PrivacyGuarantee::from_profile(profile.clone(),)]).is_err()
    );
    let first = PrivacyGuarantee::from_profile(profile.clone());
    let second = PrivacyGuarantee::from_profile(profile);
    assert!(PrivacyGuarantee::compose(vec![first, second]).is_err());

    let first = PrivacyGuarantee::from_profile(
        PrivacyProfile::new(|_| Ok(1.0)).with_approxDP(vec![(0.1, 0.2)])?,
    )
    .with_zCDP(0.1, 0.0)?;
    let second = PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![first, second])?;

    assert!(composed.profile.is_none());
    assert!((0.3..0.31).contains(&composed.zcdp.unwrap().rho));
    Ok(())
}

#[cfg(feature = "honest-but-curious")]
#[test]
fn test_tradeoff_only_composition_has_no_supported_path() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_tradeoff(|_| Ok(0.5))?;
    let second = PrivacyGuarantee::new().with_tradeoff(|_| Ok(0.4))?;
    let error = PrivacyGuarantee::compose(vec![first, second]).unwrap_err();
    assert_eq!(
        error.message.as_deref(),
        Some("PrivacyGuarantee composition has no supported common representation")
    );
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
        .with_renyiDP_trusted(
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
        .with_renyiDP_trusted(|_| Ok(0.1), 0.03)?
        .with_zCDP(0.2, 0.4)?;
    let second = PrivacyGuarantee::new().with_renyiDP_trusted(|_| Ok(0.2), 0.05)?;
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
    assert!(identity.tradeoff.is_none());
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
