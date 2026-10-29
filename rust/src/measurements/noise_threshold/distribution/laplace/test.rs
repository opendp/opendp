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
fn test_make_laplace_threshold_native_types() -> Fallible<()> {
    macro_rules! test_make_laplace_type {
        ($($ty:ty),+) => {$(
            // map
            let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<$ty>::new_non_nan());
            let metric = L0PInfDistance(AbsoluteDistance::<$ty>::default());
            let meas = make_laplace_threshold(domain, metric, 1., <$ty>::inf_cast(50)?, None)?;

            let data = HashMap::from([(false, <$ty>::zero()), (true, <$ty>::inf_cast(100)?)]);
            let release = meas.invoke(&data)?;
            assert_eq!(release.len(), 1);
            assert!(!release.contains_key(&false));
            assert!(release.contains_key(&true));
            assert_eq!(meas.map(&(1, <$ty>::one(), <$ty>::one()))?, (1.0, 1.1102230246251565e-16));
        )+}
    }

    test_make_laplace_type!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64
    );
    Ok(())
}

#[test]
fn test_make_laplace_threshold_bigint() -> Fallible<()> {
    let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<IBig>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<RBig>::default());
    let meas = make_laplace_threshold(domain, metric, 1., ibig!(50), None)?;

    let data = HashMap::from([(false, ibig!(0)), (true, ibig!(100))]);
    let release = meas.invoke(&data)?;
    assert_eq!(release.len(), 1);
    assert!(!release.contains_key(&false));
    assert!(release.contains_key(&true));
    assert_eq!(
        meas.map(&(1, rbig!(1), rbig!(1)))?,
        (1.0, 1.1102230246251565e-16)
    );
    Ok(())
}

#[test]
fn test_make_laplace_threshold_float_map() -> Fallible<()> {
    let m_float = make_laplace_threshold(
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<f64>::new_non_nan(),
        ),
        L0PInfDistance(AbsoluteDistance::<f64>::default()).clone(),
        1f64,
        10f64,
        None,
    )?;

    assert!(m_float.map(&(1, -1., -1.)).is_err());
    assert_eq!(m_float.map(&(1, -0., -0.))?, (0.0, 0.0));
    assert_eq!(m_float.map(&(1, 0., 0.))?, (0.0, 0.0));
    assert_eq!(m_float.map(&(1, 1., 1.))?, (1.0, 6.17049020433802e-5));
    assert_eq!(m_float.map(&(1, 2., 2.))?, (2.0, 0.0001677313139513137));
    assert_eq!(m_float.map(&(1, 3., 3.))?, (3.0, 0.0004559409827773475));
    assert!(
        m_float
            .map(&(1, f64::MAX, f64::MAX))
            .unwrap_err()
            .message
            .unwrap()
            .contains("must not be smaller than")
    );
    assert!(
        m_float
            .map(&(1, f64::INFINITY, f64::INFINITY))
            .unwrap_err()
            .message
            .unwrap()
            .contains("must be finite")
    );
    assert!(
        m_float
            .map(&(1, f64::NAN, f64::NAN))
            .unwrap_err()
            .message
            .unwrap()
            .contains("must be finite")
    );
    Ok(())
}

#[test]
fn test_make_laplace_threshold_int_map() -> Fallible<()> {
    let m_int = make_laplace_threshold(
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<i32>::new_non_nan(),
        ),
        L0PInfDistance(AbsoluteDistance::<u32>::default()),
        1f64,
        10,
        None,
    )?;

    // bounds don't match float case because float case uses
    // a looser continuous approximation to tail bound
    assert_eq!(m_int.map(&(1, 0, 0))?, (0.0, 0.0));
    assert_eq!(m_int.map(&(1, 1, 1))?, (1.0, 3.319000812207484e-5));
    assert_eq!(m_int.map(&(1, 2, 2))?, (2.0, 9.021979596468555e-5));
    assert_eq!(m_int.map(&(1, 3, 3))?, (3.0, 0.00024524283193794183));

    Ok(())
}

#[test]
fn test_make_laplace_threshold_extreme_int() -> Fallible<()> {
    // an extreme noise scale dominates the output, resulting in the release always being saturated
    let meas = make_laplace_threshold(
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
fn test_make_noise_threshold_zexpfamily1_large_scale() -> Fallible<()> {
    let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<IBig>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<RBig>::default());
    let distribution = ZExpFamily::<1> {
        scale: rbig!(23948285282902934157),
    };

    let meas = distribution.make_noise_threshold((domain, metric), ibig!(23948285282902934157))?;
    // random large number:
    let data = HashMap::from([(false, ibig!(0)), (true, ibig!(23948285282902934157))]);
    assert!(meas.invoke(&data).is_ok());

    let d_in = (1, rbig!(23948285282902934157), rbig!(23948285282902934157));
    assert_eq!(meas.map(&d_in)?, (1.0, 0.5));
    Ok(())
}

#[test]
fn test_make_noise_threshold_zexpfamily1_zero_scale() -> Fallible<()> {
    let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<IBig>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<RBig>::default());
    let distribution = ZExpFamily { scale: rbig!(0) };

    let meas: Measurement<_, _, Approximate<MaxDivergence>, _> =
        distribution.make_noise_threshold((domain, metric), ibig!(100))?;

    let data = HashMap::from([(false, ibig!(100)), (true, ibig!(101))]);
    let expected = HashMap::from([(true, ibig!(101))]);
    assert_eq!(meas.invoke(&data)?, expected);

    assert_eq!(meas.map(&(1, rbig!(0), rbig!(0)))?, (0.0, 0.0));
    assert_eq!(meas.map(&(1, rbig!(1), rbig!(1)))?, (f64::INFINITY, 1.));
    Ok(())
}

#[test]
fn test_laplace_threshold_int() -> Fallible<()> {
    let input_domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<i32>::default());
    let input_metric = L0PInfDistance(AbsoluteDistance::<i32>::default());
    let m_thresh =
        make_laplace_threshold(input_domain.clone(), input_metric.clone(), 0.0, 10, None)?;

    // Equality with the threshold must be excluded.
    let release = m_thresh.invoke(&HashMap::from([(false, 9), (true, 10)]))?;
    assert!(release.is_empty());
    assert_eq!(
        m_thresh.invoke(&HashMap::from([(true, 11)]))?,
        HashMap::from([(true, 11)])
    );
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
    let input_metric = L0PInfDistance(AbsoluteDistance::<i32>::default());
    // when k is None, the grid is on subnormal increments, so nothing rounds, all values are exact
    let m_thresh = make_laplace_threshold(input_domain, input_metric, 0.0, 10.0, None)?;

    let release = m_thresh.invoke(&HashMap::from([(false, 9.99999999), (true, 10.0)]))?;
    assert!(release.is_empty());
    assert_eq!(m_thresh.map(&(1, 1, 1))?, (f64::INFINITY, 1.0));
    Ok(())
}

#[test]
fn test_laplace_threshold_float_k() -> Fallible<()> {
    let input_domain = MapDomain::new(
        AtomDomain::<bool>::default(),
        AtomDomain::<f64>::new_non_nan(),
    );
    let input_metric = L0PInfDistance(AbsoluteDistance::<i32>::default());
    // k = -1 means grid is on 0.5 increments, so 9.74 rounds to 9.5 and 9.76 rounds to 10.0
    let data = HashMap::from([(false, 9.74999), (true, 9.7500001)]);
    for (threshold, releases_ten) in [(9.5, true), (9.6, true), (9.9, true), (10.0, false)] {
        let m_thresh = make_laplace_threshold(
            input_domain.clone(),
            input_metric.clone(),
            0.0,
            threshold,
            Some(-1),
        )?;
        let expected = if releases_ten {
            HashMap::from([(true, 10.0)])
        } else {
            HashMap::new()
        };
        assert_eq!(m_thresh.invoke(&data)?, expected, "threshold={threshold}");
        assert_eq!(m_thresh.map(&(1, 1, 1))?, (f64::INFINITY, 1.0));
    }
    Ok(())
}

#[test]
fn test_laplace_threshold_negative_strict() -> Fallible<()> {
    let measurement = make_laplace_threshold(
        MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<i32>::default()),
        L0PInfDistance(AbsoluteDistance::<i32>::default()),
        0.0,
        -10,
        None,
    )?;
    let release = measurement.invoke(&HashMap::from([(false, -10), (true, -11)]))?;
    assert_eq!(release, HashMap::from([(true, -11)]));
    Ok(())
}

#[test]
fn test_laplace_threshold_delta_empirical() -> Fallible<()> {
    const N: u32 = 2048;
    let domain = MapDomain::new(AtomDomain::<u32>::default(), AtomDomain::<i32>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<i32>::default());

    // Match N, scale, positive threshold, and tolerance in
    // make_private_lazyframe/group_by/test.rs::test_stable_keys_delta_empirical.
    // Each key is unstable relative to the empty map, so its release probability
    // must be at most delta. The margin controls sampling fluctuations.
    for (count, threshold) in [(1, 2), (-1, -2)] {
        let measurement =
            make_laplace_threshold(domain.clone(), metric.clone(), 0.5, threshold, None)?;
        let data: HashMap<_, _> = (0..N).map(|key| (key, count)).collect();
        let release_rate = measurement.invoke(&data)?.len() as f64 / f64::from(N);
        let delta = measurement.map(&(1, 1, 1))?.1;
        assert!(
            release_rate <= delta + 0.05,
            "release rate {release_rate} exceeds delta {delta} (threshold {threshold})"
        );
    }
    Ok(())
}

#[test]
fn test_laplace_threshold_zero_valued_keys() -> Fallible<()> {
    let domain = MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<i32>::default());
    let metric = L0PInfDistance(AbsoluteDistance::<i32>::default());

    for threshold in [-1, 1] {
        let measurement =
            make_laplace_threshold(domain.clone(), metric.clone(), 20.0, threshold, None)?;
        assert_eq!(measurement.map(&(0, 0, 0))?, (0.0, 0.0));

        // Zero-valued entries are indistinguishable from absent keys under L0PInfDistance.
        // Opposite-sign nonzero entries must still be eligible for noise.
        let mut released_opposite_sign = false;
        for _ in 0..64 {
            let result = measurement.invoke(&HashMap::from([(false, 0), (true, -threshold)]))?;
            assert!(!result.contains_key(&false));
            released_opposite_sign |= result.contains_key(&true);
        }
        assert!(
            released_opposite_sign,
            "opposite-sign values must still be sampled"
        );
    }
    Ok(())
}

#[test]
fn test_laplace_threshold_float_rounded_to_zero() -> Fallible<()> {
    let measurement = make_laplace_threshold(
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<f64>::new_non_nan(),
        ),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
        20.0,
        1.0,
        Some(0),
    )?;
    // Both values round to zero on the integer grid.
    for _ in 0..64 {
        let result = measurement.invoke(&HashMap::from([(false, 0.0), (true, 0.125)]))?;
        assert!(result.is_empty());
    }
    Ok(())
}
