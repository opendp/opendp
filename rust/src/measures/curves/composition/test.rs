use super::*;
#[cfg(feature = "contrib")]
use crate::{combinators::CompositionMeasure, measures::MultiDP};

fn rdp_epsilon(curve: &PrivacyGuarantee, alpha: f64) -> Fallible<f64> {
    (curve.renyi_dp.as_ref().unwrap().curve)(alpha)
}

#[test]
fn test_composition_retains_native_rdp_and_zcdp() -> Fallible<()> {
    let first = PrivacyGuarantee::new()
        .with_renyiDP_trusted(|_| Ok(0.5), 0.01)?
        .with_zCDP(0.1, 0.1)?;
    let second = PrivacyGuarantee::new()
        .with_renyiDP_trusted(|_| Ok(0.75), 0.02)?
        .with_zCDP(0.2, 0.2)?;

    let composed = PrivacyGuarantee::compose(vec![first, second])?;

    let rdp = composed.renyi_dp.as_ref().unwrap();
    let zcdp = composed.zcdp.unwrap();
    assert!(rdp_epsilon(&composed, 2.0)? >= 1.25);
    assert!((0.03..0.04).contains(&rdp.delta));
    assert!(zcdp.rho >= 0.3);
    assert!((0.3..0.4).contains(&zcdp.delta));
    Ok(())
}

#[test]
fn test_zcdp_and_rdp_compose_via_rdp() -> Fallible<()> {
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?;
    let rdp = PrivacyGuarantee::new().with_renyiDP_trusted(|_| Ok(0.1), 0.0)?;

    let composed = PrivacyGuarantee::compose(vec![zcdp, rdp])?;

    assert!(composed.renyi_dp.is_some());
    assert!(composed.zcdp.is_none());
    assert!(rdp_epsilon(&composed, 2.0)? >= 0.5);
    Ok(())
}

#[cfg(feature = "contrib")]
#[test]
fn test_sequential_composition_uses_guarantee_composition() -> Fallible<()> {
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?;
    let rdp = PrivacyGuarantee::new().with_renyiDP_trusted(|_| Ok(0.1), 0.0)?;

    let composed = MultiDP.compose(vec![zcdp, rdp])?;

    assert!(composed.renyi_dp.is_some());
    assert!(composed.zcdp.is_none());
    Ok(())
}

#[test]
fn test_rdp_uses_tighter_native_or_zcdp_bound_pointwise() -> Fallible<()> {
    let curve = PrivacyGuarantee::new()
        .with_renyiDP_trusted(|alpha| Ok(if alpha < 3.0 { alpha * 2.0 } else { 1.0 }), 0.0)?
        .with_zCDP(1.0, 0.0)?;

    let composed = PrivacyGuarantee::compose(vec![curve])?;

    // zCDP is tighter at order two; native RDP is tighter at order four.
    assert_eq!(rdp_epsilon(&composed, 2.0)?, 2.0);
    assert_eq!(rdp_epsilon(&composed, 4.0)?, 1.0);
    Ok(())
}

#[test]
fn test_rdp_zcdp_and_zcdp_retain_both() -> Fallible<()> {
    let both = PrivacyGuarantee::new()
        .with_renyiDP_trusted(|_| Ok(0.5), 0.0)?
        .with_zCDP(0.1, 0.0)?;
    let zcdp = PrivacyGuarantee::new().with_zCDP(0.2, 0.0)?;

    let composed = PrivacyGuarantee::compose(vec![both, zcdp])?;

    assert!(composed.renyi_dp.is_some());
    assert!(composed.zcdp.is_some());
    assert!(rdp_epsilon(&composed, 2.0)? >= 0.6);
    assert!(composed.zcdp.unwrap().rho >= 0.3);
    Ok(())
}

#[test]
#[cfg(feature = "honest-but-curious")]
fn test_profile_only_composition_has_no_supported_path() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_profile(|_| Ok(0.1))?;
    let second = PrivacyGuarantee::new().with_profile(|_| Ok(0.2))?;

    let error = PrivacyGuarantee::compose(vec![first, second])
        .err()
        .unwrap();
    assert_eq!(
        error.message.as_deref(),
        Some("PrivacyGuarantee composition has no supported common representation")
    );
    Ok(())
}

#[test]
fn test_nary_composition() -> Fallible<()> {
    let curves = [(0.1, 0.01), (0.2, 0.02), (0.3, 0.03), (0.4, 0.04)]
        .into_iter()
        .map(|(rho, epsilon)| {
            PrivacyGuarantee::new()
                .with_renyiDP_trusted(move |_| Ok(epsilon), 0.0)?
                .with_zCDP(rho, 0.0)
        })
        .collect::<Fallible<Vec<_>>>()?;

    let composed = PrivacyGuarantee::compose(curves)?;

    assert!(rdp_epsilon(&composed, 2.0)? >= 0.1);
    assert!(composed.zcdp.unwrap().rho >= 1.0);
    Ok(())
}

#[test]
fn test_rdp_callback_error_propagates() -> Fallible<()> {
    let failing = PrivacyGuarantee::new()
        .with_renyiDP_trusted(
            |_| fallible!(FailedFunction, "composition callback failed"),
            0.0,
        )?
        .with_zCDP(0.1, 0.0)?;
    let composed = PrivacyGuarantee::compose(vec![failing])?;

    let error = rdp_epsilon(&composed, 2.0).unwrap_err();
    assert_eq!(
        error.message.as_deref(),
        Some("composition callback failed")
    );
    Ok(())
}

#[test]
fn test_pure_zcdp_rho_sums_conservatively() -> Fallible<()> {
    let curves = [0.1, 0.2, 0.3]
        .into_iter()
        .map(|rho| PrivacyGuarantee::new().with_zCDP(rho, 0.0))
        .collect::<Fallible<Vec<_>>>()?;

    let composed = PrivacyGuarantee::compose(curves)?;
    let zcdp = composed.zcdp.unwrap();

    assert!(zcdp.rho >= 0.6);
    assert_eq!(zcdp.delta, 0.0);
    Ok(())
}

#[test]
fn test_approximate_zcdp_delta_is_not_embedded_into_rdp() -> Fallible<()> {
    let first = PrivacyGuarantee::new().with_zCDP(0.1, 0.01)?;
    let second = PrivacyGuarantee::new().with_zCDP(0.2, 0.02)?;

    let composed = PrivacyGuarantee::compose(vec![first, second])?;

    assert!(composed.renyi_dp.is_none());
    let zcdp = composed.zcdp.unwrap();
    assert!(zcdp.delta >= 0.03);
    Ok(())
}

#[test]
fn test_native_rdp_delta_remains_local_when_approximate_zcdp_is_also_present() -> Fallible<()> {
    let first = PrivacyGuarantee::new()
        .with_renyiDP_trusted(|_| Ok(0.1), 0.03)?
        .with_zCDP(0.2, 0.4)?;
    let second = PrivacyGuarantee::new().with_renyiDP_trusted(|_| Ok(0.2), 0.05)?;

    let composed = PrivacyGuarantee::compose(vec![first, second])?;
    let rdp = composed.renyi_dp.unwrap();

    assert!(rdp.delta >= 0.08);
    assert!(rdp.delta < 0.09);
    Ok(())
}

#[test]
fn test_composition_identity_and_singleton_approxdp() -> Fallible<()> {
    let identity = PrivacyGuarantee::compose(vec![])?;
    assert_eq!(identity.delta(0.0)?, 0.0);
    assert_eq!(identity.beta(0.25)?, 0.75);

    let first = PrivacyGuarantee::new().with_approxDP(vec![(0.1, 0.2)])?;
    let second = PrivacyGuarantee::new().with_approxDP(vec![(0.2, 0.3)])?;
    let composed = PrivacyGuarantee::compose(vec![first, second])?;

    let epsilon = composed.epsilon(0.5)?;
    assert!(epsilon >= 0.3);
    assert!(composed.delta(epsilon)? >= 0.5);
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
