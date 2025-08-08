use super::*;

use crate::{measures::ZeroConcentratedDivergence, metrics::SymmetricDistance};

#[test]
fn test_private_quantile_unsized() -> Fallible<()> {
    let candidates = vec![0, 25, 50, 75, 100];
    let input_domain = VectorDomain::new(AtomDomain::default());

    let m_q75 = make_private_quantile(
        input_domain.clone(),
        SymmetricDistance,
        ZeroConcentratedDivergence,
        candidates.clone(),
        0.75,
        0.0,
    )?;
    assert_eq!(m_q75.invoke(&(0..100).collect())?, 75);
    // since scale is zero, no noise is added, so eta is 0
    assert_eq!(m_q75.map(&1)?, f64::INFINITY);

    let m_q75 = make_private_quantile(
        input_domain,
        SymmetricDistance,
        ZeroConcentratedDivergence,
        candidates.clone(),
        0.75,
        1.0,
    )?;
    assert!((50..100).contains(&m_q75.invoke(&(0..100).collect())?));
    // (d_in * alpha / scale * 2)^2 / 8 = (1 * (3 / 4) / 1 * 2)^2 / 8
    assert_eq!(m_q75.map(&1)?, 0.28125);
    Ok(())
}

#[test]
fn test_private_quantile_sized() -> Fallible<()> {
    let candidates = vec![0, 25, 50, 75, 100];
    let input_domain = VectorDomain::new(AtomDomain::default()).with_size(100);

    let m_q75 = make_private_quantile(
        input_domain.clone(),
        SymmetricDistance,
        ZeroConcentratedDivergence,
        candidates.clone(),
        0.75,
        0.0,
    )?;
    assert_eq!(m_q75.invoke(&(0..100).collect())?, 75);
    assert_eq!(m_q75.map(&2)?, f64::INFINITY);

    let m_q75_sized = make_private_quantile(
        input_domain.clone(),
        SymmetricDistance,
        ZeroConcentratedDivergence,
        candidates,
        0.75,
        1.0,
    )?;
    assert!((50..100).contains(&m_q75.invoke(&(0..100).collect())?));
    // (d_in / scale)^2 / 8 = (2 / 1)^2 / 8 = 1 / 2
    assert_eq!(m_q75_sized.map(&2)?, 0.5);
    Ok(())
}
