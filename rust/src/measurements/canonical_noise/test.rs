use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;
use crate::error::ErrorVariant;
use dashu::{rational::RBig, rbig};

fn measurement(
    specification: PrivacyGuarantee,
) -> Fallible<Measurement<AtomDomain<f64>, AbsoluteDistance<f64>, MultiDP, f64>> {
    make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        1.0,
        specification,
    )
}

#[test]
fn test_exact_symmetry_fixed_point_and_sampled_constraints() -> Fallible<()> {
    let samples = [(0.25, 0.2), (0.7, 0.01), (2.0, 0.0)];
    let compiled = PolyhedralTradeoff::new(&samples)?;
    assert_eq!(compiled.beta(&compiled.fixed_point), compiled.fixed_point);
    assert!(compiled.fixed_point > RBig::ZERO && compiled.fixed_point < rbig!(1 / 2));
    for point in &compiled.constraints {
        let exp_oracle = rug::Float::with_val(512, point.epsilon).exp();
        assert!(rug::Float::with_val(512, point.q.to_f64().value()) <= exp_oracle);
        assert!(point.q >= RBig::ONE);
        assert!(point.fixed_point() <= compiled.fixed_point);
        for numerator in 0..=16 {
            let alpha = RBig::from(numerator) / RBig::from(16);
            let beta = compiled.beta(&alpha);
            assert!(beta >= point.beta(&alpha));
            // Both directional DP inequalities hold with q, hence also with
            // exp(epsilon) >= q, at every point of the exact envelope.
            let delta = RBig::try_from(point.delta)?;
            assert!(RBig::ONE - &beta - &point.q * &alpha <= delta);
            assert!(RBig::ONE - &alpha - &point.q * &beta <= delta);
            for second in 0..=16 {
                let other = RBig::from(second) / RBig::from(16);
                // Generalized inverse symmetry also handles zero plateaus.
                assert_eq!(beta <= other, compiled.beta(&other) <= alpha);
            }
        }
    }
    Ok(())
}

#[test]
fn test_nonmonotone_samples_use_prefix_minima() -> Fallible<()> {
    let samples = [(0.25, 0.8), (0.5, 0.2), (1.0, 0.4), (2.0, 0.1)];
    let compiled = PolyhedralTradeoff::new(&samples)?;
    for (point, sample) in compiled.constraints.iter().zip(samples) {
        assert_eq!((point.epsilon, point.delta), sample);
    }
    let profile = compiled.profile()?;
    let epsilons = [
        0.0,
        0.25,
        0.25,
        0.4,
        0.5,
        0.75,
        1.0,
        1.5,
        2.0,
        3.0,
        f64::INFINITY,
    ];
    let expected = [1.0, 0.8, 0.8, 0.8, 0.2, 0.2, 0.2, 0.2, 0.1, 0.1, 0.1];
    for (actual, expected) in profile.delta(&epsilons)?.into_iter().zip(expected) {
        assert!(actual >= expected);
        assert!(actual <= expected + 1e-14);
    }
    assert_eq!(profile.delta(&[0.0])?, vec![1.0]);
    assert_eq!(profile.delta(&[2.0, f64::INFINITY])?, vec![0.1, 0.1]);
    assert_eq!(profile.epsilon(&[0.1])?, vec![2.0]);
    assert!(profile.delta(&[])?.is_empty());
    assert_eq!(compiled.beta(&compiled.fixed_point), compiled.fixed_point);
    Ok(())
}

#[test]
fn test_certificate_endpoints_zero_and_tiny_delta() -> Fallible<()> {
    for delta in [0.0, f64::from_bits(1), 1e-300, 1e-7] {
        let specification = PrivacyGuarantee::new().with_approx_dp(0.7, delta)?;
        let compiled = compile_profile(&specification)?;
        let endpoint = compiled
            .constraints
            .iter()
            .find(|point| point.epsilon == 0.7)
            .unwrap();
        assert_eq!(endpoint.delta, delta);
        let profile = compiled.profile()?;
        assert_eq!(profile.delta(&[0.7, f64::INFINITY])?, vec![delta, delta]);
        assert_eq!(profile.epsilon(&[delta])?, vec![0.7]);
        assert_eq!(compiled.beta(&compiled.fixed_point), compiled.fixed_point);
        let measurement = measurement(specification)?;
        assert_eq!(measurement.map(&1.0)?.delta(0.7)?, delta);
        assert_eq!(
            measurement.output_measure.capabilities().profile(),
            Some(if delta == 0.0 {
                Purity::Pure
            } else {
                Purity::Approximate
            })
        );
    }
    Ok(())
}

#[test]
fn test_zero_epsilon_positive_delta() -> Fallible<()> {
    let specification = PrivacyGuarantee::new().with_approx_dp(0.0, 0.25)?;
    let compiled = compile_profile(&specification)?;
    assert_eq!(compiled.fixed_point, rbig!(3 / 8));
    assert_eq!(compiled.beta(&compiled.fixed_point), compiled.fixed_point);
    let measurement = measurement(specification)?;
    assert_eq!(measurement.map(&1.0)?.delta(0.0)?, 0.25);
    assert!(measurement.invoke(&0.0)?.is_finite());
    Ok(())
}

#[test]
fn test_extreme_and_unsupported_epsilon() -> Fallible<()> {
    // The finite inward f64 boundary is still a valid lower enclosure for an
    // exponential beyond f64's range; no infinite rational is constructed.
    for epsilon in [512.0, 1000.0] {
        let point = ApproxDPConstraint::new(epsilon, 0.0)?;
        assert!(point.q >= RBig::ONE);
        let compiled = PolyhedralTradeoff::new(&[(epsilon, 0.0)])?;
        assert_eq!(compiled.beta(&compiled.fixed_point), compiled.fixed_point);
    }
    let error = ApproxDPConstraint::new(f64::MAX, 0.1).err().unwrap();
    assert_eq!(error.variant, ErrorVariant::NumericError);
    assert!(error.message.unwrap().contains("exponential range"));
    assert!(compile_profile(&PrivacyGuarantee::new().with_approx_dp(f64::MAX, 0.1)?).is_err());
    Ok(())
}

#[test]
fn test_rejects_degenerate_and_invalid_samples() -> Fallible<()> {
    for samples in [
        vec![],
        vec![(0.0, 0.0)],
        vec![(1.0, 1.0)],
        vec![(2.0, 0.1), (1.0, 0.1)],
        vec![(f64::NAN, 0.1)],
        vec![(f64::INFINITY, 0.1)],
        vec![(-1.0, 0.1)],
        vec![(1.0, f64::NAN)],
        vec![(1.0, -0.1)],
        vec![(1.0, 1.1)],
    ] {
        assert!(PolyhedralTradeoff::new(&samples).is_err());
    }
    for specification in [
        PrivacyGuarantee::new().with_approx_dp(0.0, 0.0)?,
        PrivacyGuarantee::new().with_approx_dp(1.0, 1.0)?,
    ] {
        assert_eq!(
            measurement(specification).unwrap_err().variant,
            ErrorVariant::MakeMeasurement
        );
    }
    assert!(measurement(PrivacyGuarantee::new()).is_err());
    Ok(())
}

#[test]
fn test_compiles_once_and_returns_only_profile() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let recorded = calls.clone();
    let specification = PrivacyGuarantee::new()
        .with_profile(PrivacyProfile::new_log(move |epsilons| {
            recorded.fetch_add(epsilons.len(), Ordering::Relaxed);
            Ok(vec![-1.0; epsilons.len()])
        }))
        .with_zCDP(1.0, 0.5)?
        .with_renyiDP(|orders| Ok(vec![f64::INFINITY; orders.len()]), 0.0)?;
    #[cfg(feature = "idealized-numerics")]
    let specification = specification.with_gaussianDP(64.0)?;
    let measurement = measurement(specification)?;
    let count = calls.load(Ordering::Relaxed);
    assert_eq!(count, CANONICAL_EPSILONS.len());
    for distance in [0.0, 0.5, 1.0] {
        let output = measurement.map(&distance)?;
        assert!(output.profile().is_some());
        assert!(output.approx_dp().is_none());
        let debug = format!("{output:?}");
        for representation in ["renyi: false", "zcdp: false"] {
            assert!(debug.contains(representation));
        }
        #[cfg(feature = "idealized-numerics")]
        assert!(debug.contains("gaussian: false"));
        output.delta(1.0)?;
    }
    assert!(measurement.invoke(&1.0)?.is_finite());
    assert_eq!(calls.load(Ordering::Relaxed), count);
    let capabilities = measurement.output_measure.capabilities();
    assert_eq!(capabilities.profile(), Some(Purity::Approximate));
    assert_eq!(capabilities.approx_dp(), None);
    assert_eq!(capabilities.renyi(), None);
    assert_eq!(capabilities.zcdp(), None);
    assert!(!capabilities.gaussian());
    Ok(())
}

#[test]
fn test_sampling_and_zero_distance() -> Fallible<()> {
    let specification = PrivacyGuarantee::new().with_approx_dp(1.0, 1e-6)?;
    let measurement = measurement(specification.clone())?;
    for _ in 0..8 {
        assert!(measurement.invoke(&0.0)?.is_finite());
    }
    let zero = measurement.map(&0.0)?;
    assert_eq!(zero.delta(0.0)?, 0.0);
    assert_eq!(zero.epsilon(0.0)?, 0.0);
    assert_eq!(
        zero.profile().unwrap().delta(&[0.0, 1.0, f64::INFINITY])?,
        vec![0.0; 3]
    );
    for distance in [-1.0, f64::NAN, f64::INFINITY, 1.1] {
        assert!(measurement.map(&distance).is_err());
    }
    for distance in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(
            make_canonical_noise(
                AtomDomain::new_non_nan(),
                AbsoluteDistance::default(),
                distance,
                specification.clone()
            )
            .is_err()
        );
    }
    assert!(
        make_canonical_noise(
            AtomDomain::<f64>::default(),
            AbsoluteDistance::default(),
            1.0,
            specification.clone()
        )
        .is_err()
    );
    let zero_scale = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        0.0,
        specification,
    )?;
    assert_eq!(zero_scale.invoke(&1.0)?, 1.0);
    assert_eq!(zero_scale.map(&0.0)?.delta(0.0)?, 0.0);
    Ok(())
}

#[test]
fn test_compilation_propagates_source_errors() -> Fallible<()> {
    let specification = PrivacyGuarantee::new().with_profile(PrivacyProfile::new_log(|epsilons| {
        if epsilons.iter().any(|&epsilon| epsilon >= 0.25) {
            return fallible!(NumericError, "profile evaluation failed");
        }
        Ok(vec![-1.0; epsilons.len()])
    }));
    let error = measurement(specification).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::NumericError);
    assert_eq!(error.message.as_deref(), Some("profile evaluation failed"));
    Ok(())
}
