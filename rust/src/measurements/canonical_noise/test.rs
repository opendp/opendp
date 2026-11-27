use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::*;
use crate::{
    error::ErrorVariant,
    traits::{
        Direction,
        samplers::{InverseCDF, test::check_kolmogorov_smirnov},
    },
};

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

// Independent full symmetric envelope, used only in tests.
fn lines(samples: &[(f64, f64)]) -> Fallible<Vec<(RBig, RBig)>> {
    let mut lines = vec![(RBig::ZERO, RBig::ZERO)];
    for &(epsilon, delta) in samples {
        let q = RBig::ONE + exp_m1_lower(epsilon)?;
        let a = RBig::ONE - RBig::try_from(delta)?;
        lines.push((-&q, a.clone()));
        lines.push((-RBig::ONE / &q, a / &q));
    }
    Ok(lines)
}

fn beta(lines: &[(RBig, RBig)], u: &RBig) -> RBig {
    lines.iter().map(|(s, b)| s * u + b).max().unwrap()
}

#[test]
fn test_lower_envelope_and_profile() -> Fallible<()> {
    // Nonmonotone samples and duplicate slopes exercise hull pruning.
    let samples = [(0.25, 0.8), (0.5, 0.2), (0.5, 0.4), (1.0, 0.4), (2.0, 0.1)];
    let compiled = Arc::new(CanonicalTradeoff::new(&samples)?);
    let lines = lines(&samples)?;
    let c = &compiled.fixed_point;
    assert_eq!(beta(&lines, c), *c);
    let mut vertices = vec![RBig::ZERO, RBig::ONE, c.clone()];
    for (i, (s, a)) in lines.iter().enumerate() {
        for (t, b) in &lines[i + 1..] {
            if s != t {
                let x = (a - b) / (t - s);
                if x >= RBig::ZERO && x <= RBig::ONE {
                    vertices.push(x);
                }
            }
        }
    }
    for u in vertices.iter().filter(|u| *u <= c) {
        let index = compiled.segments.partition_point(|s| &s.start <= u);
        let actual = compiled.segments[index - 1].eval(u);
        assert_eq!(actual, RBig::ONE - beta(&lines, u) - u);
    }
    let profile = compiled.profile()?;
    let epsilons = [0.0, 0.25, 0.4, 0.5, 0.75, 1.0, 1.5, 2.0, 3.0];
    for (epsilon, actual) in epsilons.into_iter().zip(profile.delta(&epsilons)?) {
        let q = RBig::ONE + exp_m1_lower(epsilon)?;
        let expected = vertices
            .iter()
            .map(|u| RBig::ONE - beta(&lines, u) - &q * u)
            .max()
            .unwrap();
        assert!(RBig::try_from(actual)? >= expected);
        assert!(actual <= expected.to_f64().value() + 1e-14);
    }
    assert_eq!(profile.delta(&[2.0, f64::INFINITY])?, vec![0.1, 0.1]);
    assert!(profile.epsilon(&[0.1])?[0] <= 2.0);
    Ok(())
}

#[test]
fn test_certificate_endpoints_and_saturation() -> Fallible<()> {
    for (epsilon, delta) in [
        (0.0, 0.25),
        (f64::from_bits(1), 0.0),
        (1e-20, 0.0),
        (0.7, 0.0),
        (0.7, f64::from_bits(1)),
        (0.7, 1e-300),
        (0.7, 1e-7),
        (512.0, 0.0),
        (1000.0, 0.0),
        (f64::MAX, 0.1),
    ] {
        let a = exp_m1_lower(epsilon)?;
        assert!(a >= RBig::try_from(epsilon)?);
        if epsilon < 1024.0 {
            let reference = rug::Float::with_val(512, epsilon).exp_m1();
            assert!(rug::Float::with_val(512, a.to_f64().value()) <= reference);
        } else {
            assert_eq!(a, RBig::try_from(f64::MAX)?);
        }
        let m = measurement(PrivacyGuarantee::new().with_approx_dp(epsilon, delta)?)?;
        let guarantee = m.map(&1.0)?;
        assert_eq!(guarantee.delta(epsilon)?, delta);
        assert_eq!(guarantee.delta(f64::INFINITY)?, delta);
        if epsilon < 1024.0 {
            assert!(guarantee.epsilon(delta)? <= epsilon);
        }
    }
    Ok(())
}

#[test]
fn test_rejects_degenerate_and_invalid_samples() -> Fallible<()> {
    for samples in [
        vec![],
        vec![(0.0, 0.0)],
        vec![(1.0, 1.0)],
        vec![(f64::NAN, 0.1)],
        vec![(f64::INFINITY, 0.1)],
        vec![(-1.0, 0.1)],
        vec![(1.0, f64::NAN)],
        vec![(1.0, -0.1)],
        vec![(1.0, 1.1)],
    ] {
        assert!(CanonicalTradeoff::new(&samples).is_err());
    }
    for specification in [
        PrivacyGuarantee::new().with_approx_dp(0.0, 0.0)?,
        PrivacyGuarantee::new().with_approx_dp(1.0, 1.0)?,
        PrivacyGuarantee::new(),
    ] {
        assert_eq!(
            measurement(specification).unwrap_err().variant,
            ErrorVariant::MakeMeasurement
        );
    }
    Ok(())
}

#[test]
fn test_compiles_once_and_returns_only_profile() -> Fallible<()> {
    let calls = Arc::new(AtomicUsize::new(0));
    let recorded = calls.clone();
    let specification = PrivacyGuarantee::new()
        .with_profile(PrivacyProfile::new_log(move |epsilons| {
            assert_eq!(epsilons, CANONICAL_EPSILONS);
            recorded.fetch_add(1, Ordering::Relaxed);
            Ok(vec![-1.0; epsilons.len()])
        }))
        .with_zCDP(1.0, 0.5)?
        .with_renyiDP(|orders| Ok(vec![f64::INFINITY; orders.len()]), 0.0)?;
    #[cfg(feature = "idealized-numerics")]
    let specification = specification.with_gaussianDP(64.0)?;
    let measurement = measurement(specification)?;
    let count = calls.load(Ordering::Relaxed);
    assert_eq!(count, 1);
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
    Ok(())
}

#[test]
fn test_sampling_and_zero_distance() -> Fallible<()> {
    let specification = PrivacyGuarantee::new().with_approx_dp(1.0, 1e-6)?;
    let measurement = measurement(specification.clone())?;
    assert!(measurement.invoke(&0.0)?.is_finite());
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

// Deliberately unaccelerated, using the full envelope rather than the hull.
fn exact_quantile(lines: &[(RBig, RBig)], c: &RBig, mut u: RBig) -> RBig {
    if u > rbig!(1 / 2) {
        return -exact_quantile(lines, c, RBig::ONE - u);
    }
    let mut offset = RBig::ZERO;
    while &u < c {
        u = RBig::ONE - beta(lines, &u);
        offset -= RBig::ONE;
    }
    offset + (u - rbig!(1 / 2)) / (RBig::ONE - rbig!(2) * c)
}

#[test]
fn test_canonical_quantile_encloses_exact_recurrence() -> Fallible<()> {
    let samples = [(0.0, 0.2), (0.1, 0.03), (0.5, 0.0)];
    let compiled = CanonicalTradeoff::new(&samples)?;
    let lines = lines(&samples)?;
    let scale = rbig!(2 / 3);
    let rv = CanonicalRV {
        shift: rbig!(3 / 7),
        scale: &scale,
        tradeoff: &compiled,
    };
    for i in 1..32 {
        let u = RBig::from(i) / rbig!(32);
        let reference =
            exact_quantile(&lines, &compiled.fixed_point, u.clone()) * &scale + &rv.shift;
        for refinements in [0, 64, 128] {
            let lo = rv
                .inverse_cdf(u.clone(), refinements, Direction::Down)?
                .unwrap();
            let hi = rv
                .inverse_cdf(u.clone(), refinements, Direction::Up)?
                .unwrap();
            assert!(lo <= reference && reference <= hi);
            let tolerance = RBig::ONE / RBig::from(1u128 << (32 + refinements / 2));
            assert!(hi - lo < tolerance);
        }
    }
    Ok(())
}

#[test]
fn test_near_perfect_privacy_affine_tail() -> Fallible<()> {
    // Zero drift slope admits an exact translation jump.
    let compiled = CanonicalTradeoff::new(&[(0.0, 2.0_f64.powi(-80))])?;
    let rv = CanonicalRV {
        shift: RBig::ZERO,
        scale: &RBig::ONE,
        tradeoff: &compiled,
    };
    let magnitude = RBig::from(1u128 << 78);
    assert_eq!(
        rv.inverse_cdf(rbig!(1 / 4), 64, Direction::Down)?,
        Some(-&magnitude)
    );
    assert_eq!(
        rv.inverse_cdf(rbig!(3 / 4), 64, Direction::Up)?,
        Some(magnitude)
    );

    // A drift slope near 2^-60 needs about 8e17 recurrence steps.
    let compiled = CanonicalTradeoff::new(&[(2.0_f64.powi(-60), 0.0)])?;
    let rv = CanonicalRV {
        shift: RBig::ZERO,
        scale: &RBig::ONE,
        tradeoff: &compiled,
    };
    let lo = rv.inverse_cdf(rbig!(1 / 4), 64, Direction::Down)?.unwrap();
    let hi = rv.inverse_cdf(rbig!(1 / 4), 64, Direction::Up)?.unwrap();
    assert!(lo > -RBig::from(1u64 << 60));
    assert!(hi < -RBig::from(1u64 << 59));
    assert!(lo <= hi);
    assert!(hi - lo < rbig!(1 / 1000000));
    Ok(())
}

#[test]
fn test_canonical_sampling_distribution() -> Fallible<()> {
    // Evaluate the defining CDF independently of the accelerated inverse.
    fn cdf(x: f64, lines: &[(RBig, RBig)], c: f64) -> f64 {
        if x < -0.5 {
            return 1.0 - cdf(-x, lines, c);
        }
        if x > 0.5 {
            let alpha = RBig::try_from(cdf(x - 1.0, lines, c)).unwrap();
            return 1.0 - beta(lines, &alpha).to_f64().value();
        }
        0.5 + (1.0 - 2.0 * c) * x
    }

    let points = [(1.0, 1e-6)];
    let compiled = CanonicalTradeoff::new(&points)?;
    let lines = lines(&points)?;
    let c = compiled.fixed_point.to_f64().value();
    let samples = (0..5000)
        .map(|_| {
            PartialSample::new(CanonicalRV {
                shift: RBig::ZERO,
                scale: &RBig::ONE,
                tradeoff: &compiled,
            })
            .value()
        })
        .collect::<Fallible<Vec<f64>>>()?;
    check_kolmogorov_smirnov(<[f64; 5000]>::try_from(samples).unwrap(), |x| {
        cdf(x, &lines, c)
    })?;
    Ok(())
}

#[test]
fn test_small_loss_grid_refinement() -> Fallible<()> {
    let certificate = PrivacyGuarantee::new().with_approx_dp(0.001, 0.0)?;
    let profile = PrivacyProfile::new(move |epsilons| certificate.delta_batch(epsilons));
    let specification = PrivacyGuarantee::new().with_profile(profile);
    let coarse = Arc::new(CanonicalTradeoff::new(
        &CANONICAL_EPSILONS
            .iter()
            .copied()
            .zip(specification.delta_batch(CANONICAL_EPSILONS)?)
            .collect::<Vec<_>>(),
    )?);
    let refined = Arc::new(compile_profile(&specification)?);
    assert!(coarse.profile()?.delta(&[0.002])?[0] > 0.0);
    assert_eq!(refined.profile()?.delta(&[0.002])?, vec![0.0]);
    Ok(())
}
