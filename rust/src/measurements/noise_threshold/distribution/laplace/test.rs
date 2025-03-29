use std::collections::HashMap;

use crate::{
    domains::{AtomDomain, MapDomain},
    metrics::L0PI,
};

use super::*;

#[test]
fn test_laplace_threshold_int() -> Fallible<()> {
    let input_domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<i32>::default());
    let input_metric = L0PI(AbsoluteDistance::<i32>::default());
    let m_thresh =
        make_laplace_threshold(input_domain.clone(), input_metric.clone(), 0.0, 10, None)?;

    let release = m_thresh.invoke(&HashMap::from([(false, 9), (true, 10)]))?;
    assert_eq!(release, HashMap::from([(true, 10)]));
    assert_eq!(m_thresh.map(&(1, 1, 1))?, (f64::INFINITY, 1.0));

    let m_thresh = make_laplace_threshold(input_domain, input_metric, 1.0, 10, None)?;
    assert_eq!(m_thresh.map(&(1, 1, 1))?, (1.0, 3.319000812207484e-5));
    Ok(())
}

#[test]
fn test_laplace_threshold_float() -> Fallible<()> {
    let input_domain = MapDomain::new(
        AtomDomain::<bool>::default(),
        AtomDomain::<f64>::new_non_nan(),
    );
    let input_metric = L0PI(AbsoluteDistance::<i32>::default());
    // when k is None, the grid is on subnormal increments, so nothing rounds, all values are exact
    let m_thresh = make_laplace_threshold(input_domain, input_metric, 0.0, 10.0, None)?;

    let release = m_thresh.invoke(&HashMap::from([(false, 9.99999999), (true, 10.0)]))?;
    assert_eq!(release, HashMap::from([(true, 10.0)]));
    assert_eq!(m_thresh.map(&(1, 1, 1))?, (f64::INFINITY, 1.0));
    Ok(())
}

#[test]
fn test_laplace_threshold_float_k() -> Fallible<()> {
    let input_domain = MapDomain::new(
        AtomDomain::<bool>::default(),
        AtomDomain::<f64>::new_non_nan(),
    );
    let input_metric = L0PI(AbsoluteDistance::<i32>::default());
    // k = -1 means grid is on 0.5 increments, so 9.74 rounds to 9.5 and 9.76 rounds to 10.0
    let m_thresh = make_laplace_threshold(input_domain, input_metric, 0.0, 9.9, Some(-1))?;

    let release = m_thresh.invoke(&HashMap::from([(false, 9.74999), (true, 9.7500001)]))?;
    assert_eq!(release, HashMap::from([(true, 10.0)]));
    assert_eq!(m_thresh.map(&(1, 1, 1))?, (f64::INFINITY, 1.0));
    Ok(())
}
