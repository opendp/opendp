use dashu::rbig;

use crate::{
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    metrics::{L1Distance, L2Distance},
};

use super::*;

#[test]
fn test_make_noise_ibig_laplace() -> Fallible<()> {
    let space = (
        VectorDomain::new(AtomDomain::<IBig>::default()),
        L1Distance::<RBig>::default(),
    );

    assert!(
        ZExpFamily::<1> {
            scale: rbig!(-1),
            radius: None
        }
        .make_noise(space.clone())
        .map(|_: Measurement<_, _, _, MaxDivergence>| ())
        .is_err()
    );

    let m_noise: Measurement<_, _, _, MaxDivergence> = ZExpFamily::<1> {
        scale: rbig!(1),
        radius: None,
    }
    .make_noise(space.clone())?;
    assert_eq!(m_noise.map(&rbig!(1))?, 1.0);
    assert!(m_noise.invoke(&vec![IBig::from(1)]).is_ok());

    Ok(())
}

#[test]
fn test_make_noise_ibig_gaussian() -> Fallible<()> {
    let space = (
        VectorDomain::new(AtomDomain::<IBig>::default()),
        L2Distance::<RBig>::default(),
    );

    assert!(
        ZExpFamily::<2> {
            scale: rbig!(-1),
            radius: None
        }
        .make_noise(space.clone())
        .map(|_: Measurement<_, _, _, ZeroConcentratedDivergence>| ())
        .is_err()
    );

    let m_noise: Measurement<_, _, _, ZeroConcentratedDivergence> = ZExpFamily::<2> {
        scale: rbig!(1),
        radius: None,
    }
    .make_noise(space.clone())?;
    assert_eq!(m_noise.map(&rbig!(1))?, 0.5);
    assert!(m_noise.invoke(&vec![IBig::from(1)]).is_ok());

    Ok(())
}
