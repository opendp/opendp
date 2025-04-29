use crate::core::*;
use crate::domains::AtomDomain;
use crate::measurements::make_laplace;
use crate::measures::{MaxDivergence, RenyiDivergence};
use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
fn test_make_composition() -> Fallible<()> {
    let measurement0 = Measurement::new(
        AtomDomain::<i32>::default(),
        Function::new(|arg: &i32| (arg + 1) as f64),
        AbsoluteDistance::<i32>::default(),
        MaxDivergence,
        PrivacyMap::new(|_d_in: &i32| f64::INFINITY),
    )?;
    let measurement1 = Measurement::new(
        AtomDomain::<i32>::default(),
        Function::new(|arg: &i32| (arg - 1) as f64),
        AbsoluteDistance::<i32>::default(),
        MaxDivergence,
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
        Function::new(|arg| *arg),
        AbsoluteDistance::default(),
        RenyiDivergence,
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
