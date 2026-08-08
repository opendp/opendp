use crate::core::*;
use crate::domains::AtomDomain;
use crate::interactive::Queryable;
use crate::measurements::make_laplace;
use crate::measures::{Approximate, MultiDP, PrivacyGuarantee, PureDP, Purity, RenyiDP, zCDP};
use crate::metrics::{AbsoluteDistance, DiscreteDistance};

use super::*;

#[test]
fn test_make_composition() -> Fallible<()> {
    let measurement0 = Measurement::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        PureDP,
        Function::new(|arg: &i32| (arg + 1) as f64),
        PrivacyMap::new(|_d_in: &i32| f64::INFINITY),
    )?;
    let measurement1 = Measurement::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        PureDP,
        Function::new(|arg: &i32| (arg - 1) as f64),
        PrivacyMap::new(|_d_in: &i32| f64::INFINITY),
    )?;
    let composition = make_composition(vec![measurement0, measurement1])?;
    let arg = 99;
    let ret = composition.invoke(&arg)?;
    assert_eq!(ret, vec![100_f64, 98_f64]);

    Ok(())
}

#[test]
fn test_make_composition_2() -> Fallible<()> {
    let input_domain = AtomDomain::<f64>::new_non_nan();
    let input_metric = AbsoluteDistance::default();
    let laplace = make_laplace::<_, _, PureDP>(input_domain, input_metric, 1.0f64, None)?;
    let measurements = vec![laplace; 2];
    let composition = make_composition(measurements)?;
    let arg = 99.;
    let ret = composition.invoke(&arg)?;

    assert_eq!(ret.len(), 2);
    println!("return: {:?}", ret);

    assert!(composition.check(&1., &2.)?);
    assert!(!composition.check(&1., &1.9999)?);
    Ok(())
}

#[test]
fn test_rdp_composition() -> Fallible<()> {
    let m_gauss = Measurement::new(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        RenyiDP,
        Function::new(|arg| *arg),
        PrivacyMap::new(|&d_in: &f64| Function::new(move |alpha| alpha * d_in.powi(2) / 2.)),
    )?;
    let composition = make_composition(vec![m_gauss; 2])?;
    assert_eq!(composition.invoke(&2.)?, vec![2.; 2]);

    // when alpha = 3. and d_in = 2., then \bar{\epsilon} = 3. * 2.^2 / 2 = 6.
    // then we are composing two queries, so the total loss is 6. * 2. = 12.
    let rdp_curve = composition.map(&2.)?;
    assert_eq!(rdp_curve.eval(&3.0)?, 12.0);
    Ok(())
}

#[test]
fn test_multidp_composition_derives_capabilities_from_all_children() -> Fallible<()> {
    let domain = AtomDomain::<i32>::default();
    let metric = AbsoluteDistance::<i32>::default();
    let rdp_child = Measurement::new(
        domain.clone(),
        metric.clone(),
        MultiDP::with_renyi(Purity::Pure),
        Function::new(|arg: &i32| *arg),
        PrivacyMap::new_fallible(|_| {
            PrivacyGuarantee::new().with_renyiDP_trusted(|alpha| Ok(alpha), 0.0)
        }),
    )?;
    let zcdp_child = Measurement::new(
        domain,
        metric,
        MultiDP::with_zcdp(Purity::Pure),
        Function::new(|arg: &i32| *arg),
        PrivacyMap::new_fallible(|_| PrivacyGuarantee::new().with_zCDP(0.1, 0.0)),
    )?;

    let composition = make_composition(vec![rdp_child, zcdp_child])?;
    assert_eq!(
        composition.output_measure.capabilities().renyi(),
        Some(Purity::Pure)
    );
    assert_eq!(composition.output_measure.capabilities().zcdp(), None);
    assert!(composition.map(&1)?.epsilon(1e-3)?.is_finite());
    Ok(())
}

#[test]
fn test_multidp_composition_meets_zcdp_purity() -> Fallible<()> {
    let pure = Measurement::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        MultiDP::with_zcdp(Purity::Pure),
        Function::new(|arg: &i32| *arg),
        PrivacyMap::new_fallible(|_| PrivacyGuarantee::new().with_zCDP(0.1, 0.0)),
    )?;
    let approximate = Measurement::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        MultiDP::with_zcdp(Purity::Approximate),
        Function::new(|arg: &i32| *arg),
        PrivacyMap::new_fallible(|_| PrivacyGuarantee::new().with_zCDP(0.2, 0.01)),
    )?;

    let composition = make_composition(vec![pure, approximate])?;
    assert_eq!(
        composition.output_measure.capabilities().zcdp(),
        Some(Purity::Approximate)
    );
    assert_eq!(composition.output_measure.capabilities().renyi(), None);
    Ok(())
}

#[test]
fn test_multidp_composition_requires_a_common_capability_path() -> Fallible<()> {
    let make_measurement = |measure| {
        Measurement::new(
            AtomDomain::<i32>::default(),
            AbsoluteDistance::<i32>::default(),
            measure,
            Function::new(|arg: &i32| *arg),
            PrivacyMap::new(|_| PrivacyGuarantee::new()),
        )
    };
    let profile = make_measurement(MultiDP::with_profile(Purity::Pure))?;
    let tradeoff = make_measurement(MultiDP::with_tradeoff())?;

    let error = make_composition(vec![profile, tradeoff]).unwrap_err();
    assert_eq!(
        error.message.as_deref(),
        Some("MultiDP composition has no supported common capability path")
    );
    Ok(())
}

#[cfg(feature = "idealized-numerics")]
#[test]
fn test_multidp_composes_native_gaussian_dp() -> Fallible<()> {
    let make_measurement = |mu| {
        Measurement::new(
            AtomDomain::<i32>::default(),
            AbsoluteDistance::<i32>::default(),
            MultiDP::with_gaussian(),
            Function::new(|arg: &i32| *arg),
            PrivacyMap::new_fallible(move |_| PrivacyGuarantee::new().with_gaussianDP(mu)),
        )
    };
    let composition = make_composition(vec![make_measurement(0.3)?, make_measurement(0.4)?])?;
    assert!(composition.output_measure.capabilities().gaussian());
    assert!(composition.map(&1)?.delta(1.0)?.is_finite());
    Ok(())
}

#[cfg(feature = "idealized-numerics")]
#[test]
fn test_multidp_runtime_extras_do_not_enlarge_capabilities() -> Fallible<()> {
    let make_measurement = |mu| {
        Measurement::new(
            AtomDomain::<i32>::default(),
            AbsoluteDistance::<i32>::default(),
            MultiDP::with_zcdp(Purity::Pure),
            Function::new(|arg: &i32| *arg),
            PrivacyMap::new_fallible(move |_| {
                PrivacyGuarantee::new()
                    .with_zCDP(0.1, 0.0)
                    .and_then(|guarantee| guarantee.with_gaussianDP(mu))
            }),
        )
    };
    let composition = make_composition(vec![make_measurement(0.3)?, make_measurement(0.4)?])?;
    assert_eq!(
        composition.output_measure.capabilities().zcdp(),
        Some(Purity::Pure)
    );
    assert!(!composition.output_measure.capabilities().gaussian());
    assert!(composition.map(&1)?.delta(1.0)?.is_finite());
    Ok(())
}

#[cfg(feature = "idealized-numerics")]
#[test]
fn test_multidp_composition_ignores_optional_path_failure() -> Fallible<()> {
    let domain = AtomDomain::<i32>::default();
    let metric = AbsoluteDistance::<i32>::default();
    let first = Measurement::new(
        domain.clone(),
        metric.clone(),
        MultiDP::with_zcdp(Purity::Pure),
        Function::new(|arg: &i32| *arg),
        PrivacyMap::new_fallible(|_| {
            PrivacyGuarantee::new()
                .with_zCDP(0.1, 0.0)
                .and_then(|guarantee| guarantee.with_gaussianDP(f64::MAX))
        }),
    )?;
    let second = Measurement::new(
        domain,
        metric,
        MultiDP::with_zcdp(Purity::Pure),
        Function::new(|arg: &i32| *arg),
        PrivacyMap::new_fallible(|_| PrivacyGuarantee::new().with_zCDP(0.2, 0.0)),
    )?;

    let composition = make_composition(vec![first, second])?;
    assert!(composition.map(&1)?.epsilon(1e-3)?.is_finite());
    Ok(())
}

#[test]
fn test_interactive_postprocessing() -> Fallible<()> {
    let m1 = (Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(zCDP),
        Function::new_fallible(|&arg: &bool| Queryable::new_external(move |_: &()| Ok(arg))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )? >> Function::<Queryable<(), bool>, bool>::new_fallible(|qbl: &_| {
        qbl.clone().eval(&())
    }))?;

    let m2 = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(zCDP),
        Function::new(|arg: &bool| *arg),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )?;
    let mc = make_composition(vec![m1, m2])?;

    assert!(mc.invoke(&false).is_ok());
    Ok(())
}
