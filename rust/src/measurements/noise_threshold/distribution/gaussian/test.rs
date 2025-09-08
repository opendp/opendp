use core::f64;

use dashu::{ibig, integer::IBig, rbig};
use std::collections::HashMap;

use super::*;
use crate::{
    domains::{AtomDomain, MapDomain},
    metrics::{AbsoluteDistance, L0PInfDistance},
    traits::InfCast,
};
use num::{One, Zero};

#[test]
fn test_make_gaussian_threshold_native_types() -> Fallible<()> {
    macro_rules! test_make_gaussian_type {
        ($($ty:ty),+) => {$(
            // map
            let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<$ty>::new_non_nan());
            let metric = L0PInfDistance(AbsoluteDistance::<$ty>::default());
            let meas = make_gaussian_threshold(domain, metric, 1., <$ty>::inf_cast(50)?, None)?;

            let data = HashMap::from([(false, <$ty>::zero()), (true, <$ty>::inf_cast(100)?)]);
            let release = meas.invoke(&data)?;
            assert_eq!(release.len(), 1);
            assert!(!release.contains_key(&false));
            assert!(release.contains_key(&true));
            assert_eq!(meas.map(&(1, <$ty>::one(), <$ty>::one()))?, (0.5, 1.1102230246251565e-16));
        )+}
    }

    test_make_gaussian_type!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64
    );
    Ok(())
}

#[test]
fn test_make_gaussian_threshold_bigint() -> Fallible<()> {
    let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<IBig>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<RBig>::default());
    let meas = make_gaussian_threshold(domain, metric, 1., ibig!(50), None)?;

    let data = HashMap::from([(false, ibig!(0)), (true, ibig!(100))]);
    let release = meas.invoke(&data)?;
    assert_eq!(release.len(), 1);
    assert!(!release.contains_key(&false));
    assert!(release.contains_key(&true));
    assert_eq!(
        meas.map(&(1, rbig!(1), rbig!(1)))?,
        (0.5, 1.1102230246251565e-16)
    );
    Ok(())
}

#[test]
fn test_make_gaussian_threshold_map() -> Fallible<()> {
    fn test_map(map: impl Fn(&(u32, f64, f64)) -> Fallible<(f64, f64)>) -> Fallible<()> {
        assert!(map(&(1, -1., -1.)).is_err());
        assert_eq!(map(&(1, -0., -0.))?, (0.0, 0.0));
        assert_eq!(map(&(1, 0., 0.))?, (0.0, 0.0));
        assert_eq!(map(&(1, 1., 1.))?, (0.5, 1.1102230246251565e-16));
        assert_eq!(map(&(1, 2., 2.))?, (2.0, 6.661338147750939e-16));
        assert_eq!(map(&(1, 3., 3.))?, (4.5, 1.2798651027878805e-12));
        assert!(
            map(&(1, f64::MAX, f64::MAX))
                .unwrap_err()
                .message
                .unwrap()
                .contains("must not be smaller than")
        );
        assert!(
            map(&(1, f64::INFINITY, f64::INFINITY))
                .unwrap_err()
                .message
                .unwrap()
                .contains("must be finite")
        );
        assert!(
            map(&(1, f64::NAN, f64::NAN))
                .unwrap_err()
                .message
                .unwrap()
                .contains("must be finite")
        );
        Ok(())
    }

    let metric = L0PInfDistance(AbsoluteDistance::<f64>::default());
    let m_float = make_gaussian_threshold(
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<f64>::new_non_nan(),
        ),
        metric.clone(),
        1f64,
        10f64,
        None,
    )?;
    test_map(m_float.privacy_map.0.as_ref())?;

    let m_int = make_gaussian_threshold(
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<i32>::new_non_nan(),
        ),
        metric,
        1f64,
        10,
        None,
    )?;
    test_map(m_int.privacy_map.0.as_ref())?;
    Ok(())
}

#[test]
fn test_make_gaussian_threshold_extreme_int() -> Fallible<()> {
    // an extreme noise scale dominates the output, resulting in the release always being saturated
    let meas = make_gaussian_threshold(
        MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<u32>::default()),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
        f64::MAX,
        50,
        None,
    )?;

    let release = meas.invoke(&HashMap::from([(false, 0), (true, 100)]))?;
    assert!(release.len() < 3);
    Ok(())
}

#[test]
fn test_make_noise_threshold_zexpfamily2_large_scale() -> Fallible<()> {
    let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<IBig>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<RBig>::default());
    let distribution = ZExpFamily::<2> {
        scale: rbig!(23948285282902934157),
    };

    let meas = distribution.make_noise_threshold((domain, metric), ibig!(23948285282902934157))?;
    // random large number:
    let data = HashMap::from([(false, ibig!(0)), (true, ibig!(23948285282902934157))]);
    assert!(meas.invoke(&data).is_ok());

    let d_in = (1, rbig!(23948285282902934157), rbig!(23948285282902934157));
    assert_eq!(meas.map(&d_in)?, (0.5, 0.5000000596046448));
    Ok(())
}

#[test]
fn test_make_noise_threshold_zexpfamily2_zero_scale() -> Fallible<()> {
    let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<IBig>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<RBig>::default());
    let distribution = ZExpFamily { scale: rbig!(0) };

    let meas: Measurement<_, _, Approximate<ZeroConcentratedDivergence>, _> =
        distribution.make_noise_threshold((domain, metric), ibig!(100))?;

    let data = HashMap::from([(false, ibig!(0)), (true, ibig!(100))]);
    let expected = HashMap::from([(true, ibig!(100))]);
    assert_eq!(meas.invoke(&data)?, expected);

    assert_eq!(meas.map(&(1, rbig!(0), rbig!(0)))?, (0.0, 0.0));
    assert_eq!(meas.map(&(1, rbig!(1), rbig!(1)))?, (f64::INFINITY, 1.));
    Ok(())
}
