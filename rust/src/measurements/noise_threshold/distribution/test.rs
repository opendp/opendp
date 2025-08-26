use crate::{
    domains::{AtomDomain, MapDomain},
    error::Fallible,
    measurements::{make_laplace_threshold, make_noise_threshold},
    measures::{Approximate, MaxDivergence},
    metrics::L0PInfDistance,
};

#[test]
fn test_noise_threshold() -> Fallible<()> {
    let m_noise = make_noise_threshold(
        MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<i32>::default()),
        L0PInfDistance::default(),
        Approximate(MaxDivergence),
        1.0,
        10,
        None,
    )?;

    let m_lap = make_laplace_threshold(
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<i32>::new_non_nan(),
        ),
        L0PInfDistance::default(),
        1f64,
        10,
        None,
    )?;

    assert_eq!(m_noise.map(&(1, 0, 0))?, m_lap.map(&(1, 0, 0))?);
    assert_eq!(m_noise.map(&(1, 1, 1))?, m_lap.map(&(1, 1, 1))?);
    assert_eq!(m_noise.map(&(1, 2, 2))?, m_lap.map(&(1, 2, 2))?);
    assert_eq!(m_noise.map(&(1, 3, 3))?, m_lap.map(&(1, 3, 3))?);

    Ok(())
}
