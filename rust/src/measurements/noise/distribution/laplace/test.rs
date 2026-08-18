use core::f64;

use dashu::{ibig, integer::IBig, rbig};

use super::*;
use crate::{
    core::Measurement,
    domains::{AtomDomain, VectorDomain},
    measurements::NoisePrivacyMap,
    measures::{MultiDP, PureDP, Purity},
    metrics::{AbsoluteDistance, L1Distance},
    traits::samplers::test::check_kolmogorov_smirnov,
};
use num::{One, Zero};

#[test]
fn test_make_laplace_native_types() -> Fallible<()> {
    macro_rules! test_make_laplace_type {
        ($($ty:ty),+) => {$(
            // scalar
            let meas = make_laplace::<_, _, PureDP>(AtomDomain::<$ty>::new_non_nan(), AbsoluteDistance::<$ty>::default(), 1., None)?;
            meas.invoke(&<$ty>::zero())?; // checking to see if invoke works
            assert_eq!(meas.map(&<$ty>::one())?, 1.0);
            // vector
            let meas = make_laplace::<_, _, PureDP>(VectorDomain::new(AtomDomain::<$ty>::new_non_nan()), L1Distance::<$ty>::default(), 1., None)?;
            meas.invoke(&vec![<$ty>::zero()])?; // checking to see if invoke works
            assert_eq!(meas.map(&<$ty>::one())?, 1.0);
        )+}
    }

    test_make_laplace_type!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64
    );
    Ok(())
}

#[test]
fn test_make_laplace_bigint() -> Fallible<()> {
    // scalar ibig
    let meas = make_laplace::<_, _, PureDP>(
        AtomDomain::<IBig>::default(),
        AbsoluteDistance::<RBig>::default(),
        1.,
        None,
    )?;
    meas.invoke(&IBig::ZERO)?; // checking to see if invoke works
    assert_eq!(meas.map(&RBig::ONE)?, 1.0);
    // vector ibig
    let meas = make_laplace::<_, _, PureDP>(
        VectorDomain::new(AtomDomain::<IBig>::default()),
        L1Distance::<RBig>::default(),
        1.,
        None,
    )?;
    meas.invoke(&vec![IBig::ZERO])?; // checking to see if invoke works
    assert_eq!(meas.map(&RBig::ONE)?, 1.0);
    Ok(())
}

#[test]
fn test_make_laplace_kolmogorov_smirnov() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::<f64>::new_non_nan());
    let input_metric = L1Distance::<f64>::default();
    let meas = make_laplace::<_, _, PureDP>(input_domain, input_metric, 1.0, None)?;
    let samples = <[f64; 5000]>::try_from(meas.invoke(&vec![0.0; 5000])?).unwrap();

    pub fn laplace_cdf(x: f64) -> f64 {
        match x {
            x if x < 0.0 => 0.5 * (x).exp(),
            _ => 1.0 - 0.5 * (-x).exp(),
        }
    }

    check_kolmogorov_smirnov(samples, laplace_cdf)
}

#[test]
fn test_make_laplace_map() -> Fallible<()> {
    fn test_map(map: impl Fn(&f64) -> Fallible<f64>) -> Fallible<()> {
        assert!(map(&-1.).is_err());
        assert_eq!(map(&-0.)?, 0.0);
        assert_eq!(map(&0.)?, 0.0);
        assert_eq!(map(&1.)?, 1.0);
        assert_eq!(map(&2.)?, 2.0);
        assert_eq!(map(&3.)?, 3.0);
        assert_eq!(map(&f64::MAX)?, f64::MAX);
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

    let m_float = make_laplace::<_, _, PureDP>(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        1f64,
        None,
    )?;
    test_map(m_float.privacy_map.0.as_ref())?;

    let m_int = make_laplace::<_, _, PureDP>(
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<f64>::default(),
        1f64,
        None,
    )?;
    test_map(m_int.privacy_map.0.as_ref())?;
    Ok(())
}

#[test]
fn test_make_laplace_extreme_int() -> Fallible<()> {
    // an extreme noise scale dominates the output, resulting in the release always being saturated
    let meas = make_laplace::<_, _, PureDP>(
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
fn test_make_noise_zexpfamily1_large_scale() -> Fallible<()> {
    let space = (AtomDomain::<IBig>::default(), AbsoluteDistance::default());
    let distribution = ZExpFamily::<1> {
        scale: rbig!(23948285282902934157),
    };

    let meas: Measurement<_, _, PureDP, IBig> = distribution.make_noise(space)?;
    // random large number:
    assert!(i8::try_from(meas.invoke(&ibig!(0))?).is_err());
    assert_eq!(meas.map(&rbig!(23948285282902934157))?, 1.0);
    Ok(())
}

#[test]
fn test_make_noise_zexpfamily1_zero_scale() -> Fallible<()> {
    let domain = VectorDomain::<AtomDomain<IBig>>::default();
    let metric = L1Distance::default();
    let distribution = ZExpFamily { scale: rbig!(0) };

    let meas: Measurement<_, _, PureDP, Vec<IBig>> = distribution.make_noise((domain, metric))?;
    assert_eq!(meas.invoke(&vec![ibig!(0)])?, vec![ibig!(0)]);
    assert_eq!(meas.map(&rbig!(0))?, 0.);
    assert_eq!(meas.map(&rbig!(1))?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_discrete_laplace_rational_intervals_enclose_inputs() -> Fallible<()> {
    let f64_one = RBig::try_from(1.0)?;
    let f64_next = RBig::try_from(f64::from_bits(0x3ff0000000000001))?;
    let halfway = (f64_one + f64_next) / rbig!(2);
    let subnormal_half = RBig::try_from(f64::from_bits(1))? / rbig!(2);
    let huge_ratio =
        rbig!(123456789012345678901234567890123456789) / rbig!(98765432109876543210987654321);

    for value in [halfway, subnormal_half, huge_ratio] {
        let interval = rational_interval(&value)?;
        let lower = interval.lower_f64()?;
        let upper = interval.upper_f64()?;
        if lower.is_finite() {
            assert!(RBig::try_from(lower)? <= value);
        }
        if upper.is_finite() {
            assert!(value <= RBig::try_from(upper)?);
        }
    }
    Ok(())
}

#[test]
fn test_discrete_laplace_zero_scale_multidp_rejected_at_construction() -> Fallible<()> {
    let distribution = ZExpFamily::<1> { scale: rbig!(0) };
    let result: Fallible<Measurement<_, _, MultiDP, Vec<IBig>>> =
        distribution.clone().make_noise((
            VectorDomain::<AtomDomain<IBig>>::default(),
            L1Distance::default(),
        ));
    assert!(result.is_err());

    let result = <ZExpFamily<1> as NoisePrivacyMap<L1Distance<RBig>, MultiDP>>::noise_privacy_map(
        &distribution,
        &L1Distance::default(),
        &MultiDP::default(),
    );
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_discrete_laplace_rdp_fallback_handles_nonfinite_intermediates() -> Fallible<()> {
    let scale = rbig!(1) / rbig!(10).pow(1000);
    let sensitivity = scale.clone();
    let distribution = ZExpFamily::<1> { scale };
    let privacy_map =
        <ZExpFamily<1> as NoisePrivacyMap<L1Distance<RBig>, MultiDP>>::noise_privacy_map(
            &distribution,
            &L1Distance::default(),
            &MultiDP::default(),
        )?;

    let guarantee = privacy_map.eval(&sensitivity)?;
    assert!(guarantee.epsilon(0.0)?.is_finite());
    Ok(())
}

#[test]
fn test_discrete_laplace_multidp_advertises_each_representation() -> Fallible<()> {
    let distribution = DiscreteLaplace {
        scale: 2.0,
        k: None,
    };
    let measurement: Measurement<_, _, MultiDP, Vec<IBig>> = distribution.make_noise((
        VectorDomain::new(AtomDomain::<IBig>::default()),
        L1Distance::default(),
    ))?;
    let capabilities = measurement.output_measure.capabilities();

    assert_eq!(capabilities.profile(), Some(Purity::Pure));
    assert_eq!(capabilities.renyi(), Some(Purity::Pure));
    assert_eq!(capabilities.zcdp(), Some(Purity::Pure));
    assert!(!capabilities.tradeoff());
    assert!(!capabilities.gaussian());

    let guarantee = measurement.map(&rbig!(1))?;
    assert_eq!(guarantee.delta(0.5)?, 0.0);
    assert_eq!(guarantee.epsilon(0.0)?, 0.5);
    assert!(guarantee.beta(-0.1).is_err());
    assert!(guarantee.beta(f64::NAN).is_err());
    let epsilon = rbig!(1) / rbig!(2);
    let sensitivity = rbig!(1);
    let scale = rbig!(2);
    assert!(zcdp_discrete_laplace(&epsilon, &sensitivity, &scale)? >= 0.0);
    assert!(rdp_discrete_laplace(2.0, &sensitivity, &scale)? >= 0.0);
    let zero = measurement.map(&rbig!(0))?;
    assert_eq!(zero.delta(0.0)?, 0.0);
    assert_eq!(zero.epsilon(0.0)?, 0.0);
    assert_eq!(zero.beta(0.5)?, 0.5);
    Ok(())
}

#[test]
fn test_discrete_laplace_rdp_falls_back_to_pure_dp() -> Fallible<()> {
    let distribution = ZExpFamily::<1> {
        scale: rbig!(1) / rbig!(1000),
    };
    let privacy_map =
        <ZExpFamily<1> as NoisePrivacyMap<L1Distance<RBig>, MultiDP>>::noise_privacy_map(
            &distribution,
            &L1Distance::default(),
            &MultiDP::default(),
        )?;
    let sensitivity = rbig!(1) / rbig!(10);
    let _guarantee = privacy_map.eval(&sensitivity)?;

    // a = 1 / scale = 1000 makes the tight expression fail conservatively,
    // while epsilon = 100 keeps the exact pure-DP RDP fallback representable.
    let sensitivity = rbig!(1) / rbig!(10);
    let scale = rbig!(1) / rbig!(1000);
    assert!(rdp_discrete_laplace(2.0, &sensitivity, &scale).is_err());
    assert!(rdp_from_pureDP(2.0, 100.0)?.is_finite());
    Ok(())
}
