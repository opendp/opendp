use crate::metrics::AbsoluteDistance;

use super::*;

#[test]
fn test_make_noise_floatexpfamily() -> Fallible<()> {
    let space = (
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<f64>::new_non_nan(),
        ),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
    );

    assert!(
        FloatExpFamily::<1> { scale: 1.0, k: 0 }
            .make_noise_threshold(space.clone(), 10.0)
            .is_ok()
    );
    assert!(
        FloatExpFamily::<1> {
            scale: f64::NAN,
            k: 0
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_err()
    );
    let space = (
        MapDomain::new(
            AtomDomain::<bool>::default(),
            AtomDomain::<f64>::new_non_nan(),
        ),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
    );
    assert!(
        FloatExpFamily::<2> {
            scale: 1.0,
            k: i32::MIN
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_err()
    );

    assert!(
        FloatExpFamily::<2> {
            scale: 1.0,
            k: i32::MAX
        }
        .make_noise_threshold(space.clone(), 10.0)
        .is_ok()
    );

    Ok(())
}

#[test]
fn test_make_float_to_bigint_threshold_rejects_negative_distance() -> Fallible<()> {
    fn check<const P: usize>() -> Fallible<()> {
        let space = (
            MapDomain::new(
                AtomDomain::<bool>::default(),
                AtomDomain::<f64>::new_non_nan(),
            ),
            L0PInfDistance::<P, AbsoluteDistance<f64>>::default(),
        );
        let t = make_float_to_bigint_threshold(space, 10.0, 0)?;

        for d_in in [(16, -0.5, 1.0), (16, 1.0, -0.5)] {
            let err = t.map(&d_in).unwrap_err();
            assert!(err.message.unwrap().contains("-0.5"));
        }
        assert!(t.map(&(16, 0.0, 0.0)).is_ok());
        assert!(t.map(&(16, -0.0, -0.0)).is_ok());
        Ok(())
    }

    check::<1>()?;
    check::<2>()
}

#[test]
fn test_float_threshold_after_rounding() -> Fallible<()> {
    let native = then_deintegerize_hashmap::<bool, f64>(-54)?;
    let converted = native.eval(&HashMap::from([
        (false, IBig::from(1u64 << 54) + IBig::ONE),
        (true, IBig::from(1u64 << 54) + IBig::from(8u8)),
    ]))?;
    assert_eq!(converted[&false], 1.0);
    let filtered = then_filter_strict_threshold(1.0f64, false).eval(&converted)?;
    assert!(!filtered.contains_key(&false));
    assert!(filtered[&true] > 1.0);
    Ok(())
}
