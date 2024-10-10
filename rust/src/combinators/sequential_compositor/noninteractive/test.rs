use crate::core::*;
use crate::domains::AtomDomain;
use crate::measurements::make_laplace;
use crate::measures::MaxDivergence;
use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
fn test_make_basic_composition() -> Fallible<()> {
    let input_domain0 = AtomDomain::<i32>::default();
    let function0 = Function::new(|arg: &i32| (arg + 1) as f64);
    let input_metric0 = AbsoluteDistance::<i32>::default();
    let output_measure0 = MaxDivergence::default();
    let privacy_map0 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
    let measurement0 = Measurement::new(
        input_domain0,
        function0,
        input_metric0,
        output_measure0,
        privacy_map0,
    )?;
    let input_domain1 = AtomDomain::<i32>::default();
    let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
    let input_metric1 = AbsoluteDistance::<i32>::default();
    let output_measure1 = MaxDivergence::default();
    let privacy_map1 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
    let measurement1 = Measurement::new(
        input_domain1,
        function1,
        input_metric1,
        output_measure1,
        privacy_map1,
    )?;
    let composition = make_basic_composition(vec![measurement0, measurement1])?;
    let arg = 99;
    let ret = composition.invoke(&arg)?;
    assert_eq!(ret, vec![100_f64, 98_f64]);

    Ok(())
}

#[test]
fn test_make_basic_composition_2() -> Fallible<()> {
    let input_domain = AtomDomain::default();
    let input_metric = AbsoluteDistance::default();
    let laplace = make_laplace(input_domain, input_metric, 1.0f64, None)?;
    let measurements = vec![laplace; 2];
    let composition = make_basic_composition(measurements)?;
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
        AtomDomain::default(),
        Function::new(|arg| *arg),
        AbsoluteDistance::default(),
        RenyiDivergence,
        PrivacyMap::new(|&d_in: &f64| Function::new(move |alpha| alpha * d_in.powi(2) / 2.)),
    )?;
    let composition = make_basic_composition(vec![m_gauss; 2])?;
    assert_eq!(composition.invoke(&2.)?, vec![2.; 2]);

    // when alpha = 3. and d_in = 2., then \bar{\epsilon} = 3. * 2.^2 / 2 = 6.
    // then we are composing two queries, so the total loss is 6. * 2. = 12.
    let rdp_curve = composition.map(&2.)?;
    assert_eq!(rdp_curve.eval(&3.0)?, 12.0);
    Ok(())
}
