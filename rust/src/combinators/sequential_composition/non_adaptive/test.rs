use crate::core::*;
use crate::domains::AtomDomain;
use crate::interactive::Queryable;
use crate::measurements::make_laplace;
use crate::measures::{Approximate, MaxDivergence, RenyiDivergence, ZeroConcentratedDivergence};
use crate::metrics::{AbsoluteDistance, DiscreteDistance};

use super::*;

#[test]
fn test_make_composition() -> Fallible<()> {
    let measurement0 = Measurement::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        MaxDivergence,
        Function::new(|arg: &i32| (arg + 1) as f64),
        PrivacyMap::new(|_d_in: &i32| f64::INFINITY),
    )?;
    let measurement1 = Measurement::new(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<i32>::default(),
        MaxDivergence,
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
    let laplace = make_laplace::<_, _, MaxDivergence>(input_domain, input_metric, 1.0f64, None)?;
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
        RenyiDivergence,
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
fn test_interactive_postprocessing() -> Fallible<()> {
    let m1 = (Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        Function::new_fallible(|&arg: &bool| Queryable::new_external(move |_: &()| Ok(arg))),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )? >> Function::<Queryable<(), bool>, bool>::new_fallible(|qbl: &_| {
        qbl.clone().eval(&())
    }))?;

    let m2 = Measurement::new(
        AtomDomain::<bool>::default(),
        DiscreteDistance,
        Approximate(ZeroConcentratedDivergence),
        Function::new(|arg: &bool| *arg),
        PrivacyMap::new(|_| (1.0, 1e-7)),
    )?;
    let mc = make_composition(vec![m1, m2])?;

    assert!(mc.invoke(&false).is_ok());
    Ok(())
}
