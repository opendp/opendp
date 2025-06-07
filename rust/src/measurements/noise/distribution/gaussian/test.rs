use core::f64;

use dashu::{ibig, integer::IBig, rbig};
use statrs::function::erf;

use super::*;
use crate::{
    domains::{AtomDomain, VectorDomain},
    metrics::{AbsoluteDistance, L2Distance},
    traits::samplers::test::check_kolmogorov_smirnov,
};
use num::{One, Zero};

#[test]
fn test_make_gaussian_native_types() -> Fallible<()> {
    macro_rules! test_make_gaussian_type {
        ($($ty:ty),+) => {$(
            // scalar
            let meas = make_gaussian(AtomDomain::<$ty>::new_non_nan(), AbsoluteDistance::<$ty>::default(), 1., None)?;
            meas.invoke(&<$ty>::zero())?; // checking to see if invoke works
            assert_eq!(meas.map(&<$ty>::one())?, 0.5);
            // vector
            let meas = make_gaussian(VectorDomain::new(AtomDomain::<$ty>::new_non_nan()), L2Distance::<$ty>::default(), 1., None)?;
            meas.invoke(&vec![<$ty>::zero()])?; // checking to see if invoke works
            assert_eq!(meas.map(&<$ty>::one())?, 0.5);
        )+}
    }

    test_make_gaussian_type!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64
    );
    Ok(())
}

#[test]
fn test_make_gaussian_bigint() -> Fallible<()> {
    // scalar ibig
    let meas = make_gaussian(
        AtomDomain::<IBig>::default(),
        AbsoluteDistance::<RBig>::default(),
        1.,
        None,
    )?;
    meas.invoke(&IBig::ZERO)?; // checking to see if invoke works
    assert_eq!(meas.map(&RBig::ONE)?, 0.5);
    // vector ibig
    let meas = make_gaussian(
        VectorDomain::new(AtomDomain::<IBig>::default()),
        L2Distance::<RBig>::default(),
        1.,
        None,
    )?;
    meas.invoke(&vec![IBig::ZERO])?; // checking to see if invoke works
    assert_eq!(meas.map(&RBig::ONE)?, 0.5);
    Ok(())
}

#[test]
fn test_make_gaussian_kolmogorov_smirnov() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::<f64>::new_non_nan());
    let input_metric = L2Distance::<f64>::default();
    let meas = make_gaussian(input_domain, input_metric, 1.0, None)?;
    let samples = <[f64; 1000]>::try_from(meas.invoke(&vec![0.0; 1000])?).unwrap();

    pub fn normal_cdf(x: f64) -> f64 {
        (erf::erf(x / std::f64::consts::SQRT_2) + 1.0) / 2.0
    }

    check_kolmogorov_smirnov(samples, normal_cdf)
}

#[test]
fn test_make_gaussian_map() -> Fallible<()> {
    fn test_map(map: impl Fn(&f64) -> Fallible<f64>) -> Fallible<()> {
        assert!(map(&-1.).is_err());
        assert_eq!(map(&-0.)?, 0.0);
        assert_eq!(map(&0.)?, 0.0);
        assert_eq!(map(&1.)?, 0.5);
        assert_eq!(map(&2.)?, 2.0);
        assert_eq!(map(&3.)?, 4.5);
        assert_eq!(map(&f64::MAX)?, f64::INFINITY);
        assert!(
            map(&f64::INFINITY)
                .unwrap_err()
                .message
                .unwrap()
                .contains("must be finite")
        );
        assert!(
            map(&f64::NAN)
                .unwrap_err()
                .message
                .unwrap()
                .contains("must be finite")
        );
        Ok(())
    }

    let m_float = make_gaussian(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        1f64,
        None,
    )?;
    test_map(m_float.privacy_map.0.as_ref())?;

    let m_int = make_gaussian(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<f64>::default(),
        1f64,
        None,
    )?;
    test_map(m_int.privacy_map.0.as_ref())?;
    Ok(())
}

#[test]
fn test_make_gaussian_extreme_int() -> Fallible<()> {
    // an extreme noise scale dominates the output, resulting in the release always being saturated
    let meas = make_gaussian(
        AtomDomain::<u32>::default(),
        AbsoluteDistance::<f64>::default(),
        f64::MAX,
        None,
    )?;
    assert!([0, u32::MAX].contains(&meas.invoke(&0)?));

    // the smallest positive subnormal is the smallest value that can be represented
    let min_sub = f64::from_bits(1);
    // it is subnormal, and less than the smallest positive normal
    assert!(min_sub.is_subnormal() && min_sub < f64::MIN_POSITIVE);
    // (min_sub/f64::MAX)^2^2 would typically underflow, but here it saturates at min_sub
    assert_eq!(meas.map(&min_sub)?, min_sub);
    Ok(())
}

#[test]
fn test_make_noise_zexpfamily2_large_scale() -> Fallible<()> {
    let space = (AtomDomain::<IBig>::default(), AbsoluteDistance::default());
    let distribution = ZExpFamily::<2> {
        scale: rbig!(23948285282902934157),
    };

    let meas = distribution.make_noise(space)?;
    // random large number:
    assert!(i8::try_from(meas.invoke(&ibig!(0))?).is_err());
    assert_eq!(meas.map(&rbig!(23948285282902934157))?, 0.5);
    Ok(())
}

#[test]
fn test_make_noise_zexpfamily2_zero_scale() -> Fallible<()> {
    let domain = VectorDomain::<AtomDomain<IBig>>::default();
    let metric = L2Distance::default();
    let distribution = ZExpFamily { scale: rbig!(0) };

    let meas = distribution.make_noise((domain, metric))?;
    assert_eq!(meas.invoke(&vec![ibig!(0)])?, vec![ibig!(0)]);
    assert_eq!(meas.map(&rbig!(0))?, 0.);
    assert_eq!(meas.map(&rbig!(1))?, f64::INFINITY);
    Ok(())
}
