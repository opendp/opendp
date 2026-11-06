use crate::error::Fallible;

#[test]
fn test_amplify_without_replacement() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::measures::{Approximate, MaxDivergence};
    use crate::metrics::ChangeOneDistance;

    let map = PrivacyMap::new(|d: &u32| *d as f64);
    let amplified = amplify_without_replacement(ChangeOneDistance, MaxDivergence, map, 100, 10)?;

    assert_eq!(amplified.eval(&0)?, 0.0);
    assert!((amplified.eval(&1)? - (0.1 * 1.0f64.exp_m1()).ln_1p()).abs() < 1e-12);
    assert!((amplified.eval(&2)? - 2.0 * amplified.eval(&1)?).abs() < 1e-12);
    assert!((amplified.eval(&10)? - 10.0 * amplified.eval(&1)?).abs() < 1e-12);

    let map = PrivacyMap::new(|d: &u32| (*d as f64, 0.01 * (*d as f64)));
    let amplified =
        amplify_without_replacement(ChangeOneDistance, Approximate(MaxDivergence), map, 100, 10)?;
    let (epsilon, delta) = amplified.eval(&1)?;
    assert!((epsilon - (0.1 * 1.0f64.exp_m1()).ln_1p()).abs() < 1e-12);
    assert!((delta - 0.001).abs() < 1e-12);
    assert_eq!(amplified.eval(&0)?, (0.0, 0.0));
    let (epsilon_2, delta_2) = amplified.eval(&2)?;
    assert!((epsilon_2 - 2.0 * epsilon).abs() < 1e-12);
    assert!((delta_2 - delta * (1.0 + epsilon.exp())).abs() < 1e-12);
    let (epsilon_3, delta_3) = amplified.eval(&3)?;
    assert!((epsilon_3 - 3.0 * epsilon).abs() < 1e-12);
    assert!((delta_3 - delta * (1.0 + epsilon.exp() + (2.0 * epsilon).exp())).abs() < 1e-12);
    // A huge group distance overflows before its delta bound can be computed.
    assert_eq!(
        amplified.eval(&u32::MAX).unwrap_err().variant,
        crate::error::ErrorVariant::Overflow
    );

    assert!(
        amplify_without_replacement(
            ChangeOneDistance,
            MaxDivergence,
            PrivacyMap::new(|_: &u32| 1.0),
            10,
            11,
        )
        .is_err()
    );
    assert!(
        amplify_without_replacement(
            ChangeOneDistance,
            MaxDivergence,
            PrivacyMap::new(|_: &u32| 1.0),
            10,
            0,
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn test_amplify_without_replacement_boundaries() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::measures::{Approximate, MaxDivergence};
    use crate::metrics::ChangeOneDistance;

    // With no subsampling, the neighboring guarantee is unchanged.
    let no_sampling = amplify_without_replacement(
        ChangeOneDistance,
        MaxDivergence,
        PrivacyMap::new(|d: &u32| *d as f64),
        10,
        10,
    )?;
    assert!((no_sampling.eval(&1)? - 1.0).abs() < 1e-12);
    assert!((no_sampling.eval(&2)? - 2.0).abs() < 1e-12);

    // Zero epsilon and positive delta use the linear group-privacy bound.
    let zero_epsilon = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (0.0, 0.02)),
        10,
        5,
    )?;
    let (epsilon, delta) = zero_epsilon.eval(&3)?;
    assert_eq!(epsilon, 0.0);
    assert!((delta - 0.03).abs() < 1e-12);

    // For large enough groups, delta saturates at one.
    let saturated = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (1.0, 0.5)),
        3,
        3,
    )?;
    assert_eq!(saturated.eval(&3)?.1, 1.0);
    Ok(())
}

#[test]
fn test_amplify_tiny_sampling_rate() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::measures::MaxDivergence;
    use crate::metrics::ChangeOneDistance;

    let amplified = amplify_without_replacement(
        ChangeOneDistance,
        MaxDivergence,
        PrivacyMap::new(|_: &u32| 1.0),
        1_000_000_000,
        1,
    )?;
    let epsilon = amplified.eval(&1)?;

    // Ceiling of a 110-digit Decimal reference for ln(1 + (exp(1) - 1) / 10^9).
    let reference = 1.718_281_826_982_799e-9;
    assert!(epsilon.is_finite() && epsilon >= reference);
    assert!(epsilon <= reference * (1.0 + 1e-12));
    Ok(())
}

#[test]
fn test_amplify_tiny_positive_epsilon() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::measures::{Approximate, MaxDivergence};
    use crate::metrics::ChangeOneDistance;

    let amplified = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (1e-12, 1e-4)),
        2,
        1,
    )?;
    let (epsilon, delta) = amplified.eval(&10)?;

    // Upper-rounded 110-digit Decimal references using exact f64 inputs.
    let epsilon_reference = 5.000_000_000_001_25e-12;
    let delta_reference = 0.000_500_000_000_001_125_1;
    assert!(epsilon.is_finite() && epsilon >= epsilon_reference);
    assert!(delta.is_finite() && delta >= delta_reference);
    assert!(epsilon <= epsilon_reference * (1.0 + 1e-12));
    assert!(delta <= delta_reference * (1.0 + 1e-12));
    Ok(())
}

#[test]
fn test_amplify_zero_delta_positive_epsilon() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::measures::{Approximate, MaxDivergence};
    use crate::metrics::ChangeOneDistance;

    let amplified = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (1.0, 0.0)),
        2,
        1,
    )?;
    let (epsilon, delta) = amplified.eval(&25)?;

    // Ceiling of a 110-digit Decimal reference for 25 * ln(1 + expm1(1) / 2).
    let reference = 15.502_862_673_956_939;
    assert!(epsilon.is_finite() && epsilon >= reference);
    assert!(epsilon <= reference * (1.0 + 1e-12));
    assert_eq!(delta, 0.0);
    Ok(())
}

#[test]
fn test_amplify_numerical_overflow() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::measures::{Approximate, MaxDivergence};
    use crate::metrics::ChangeOneDistance;

    // Intermediate overflow must remain distinguishable from a computed
    // delta bound that legitimately saturates at one.
    let amplified = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (50.0, 1e-12)),
        2,
        1,
    )?;
    assert_eq!(
        amplified.eval(&100).unwrap_err().variant,
        crate::error::ErrorVariant::Overflow
    );

    // The exact delta bound can even be much smaller than one, despite an
    // overflowing intermediate expm1; returning delta=1 would lose this fact.
    assert_eq!(
        super::group_privacy_approx((1.0, 1e-320), 711)
            .unwrap_err()
            .variant,
        crate::error::ErrorVariant::Overflow
    );

    // A finite epsilon beyond the expm1 range is a typed overflow,
    // not a valid infinite budget.
    let large = amplify_without_replacement(
        ChangeOneDistance,
        MaxDivergence,
        PrivacyMap::new(|_: &u32| 1_000.0),
        2,
        1,
    );
    assert_eq!(
        large.err().expect("epsilon=1000 must overflow").variant,
        crate::error::ErrorVariant::Overflow
    );
    Ok(())
}

#[test]
fn test_amplify_privacy_parameter_validation() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::error::ErrorVariant;
    use crate::measures::{Approximate, MaxDivergence};
    use crate::metrics::ChangeOneDistance;

    for epsilon in [-1.0, f64::NEG_INFINITY, f64::NAN] {
        let result = amplify_without_replacement(
            ChangeOneDistance,
            MaxDivergence,
            PrivacyMap::new(move |_: &u32| epsilon),
            10,
            5,
        );
        assert_eq!(
            result.err().expect("invalid epsilon").variant,
            ErrorVariant::FailedMap
        );
    }

    for delta in [-0.1, 1.01, f64::NAN, f64::INFINITY] {
        let result = amplify_without_replacement(
            ChangeOneDistance,
            Approximate(MaxDivergence),
            PrivacyMap::new(move |_: &u32| (0.0, delta)),
            10,
            5,
        );
        assert_eq!(
            result.err().expect("invalid delta").variant,
            ErrorVariant::FailedMap
        );
    }

    // Epsilon and delta may be negative zero, which is normalized to +0.
    // Delta one and full-population sampling are valid boundary cases.
    for epsilon in [0.0, -0.0] {
        let pure = amplify_without_replacement(
            ChangeOneDistance,
            MaxDivergence,
            PrivacyMap::new(move |_: &u32| epsilon),
            10,
            10,
        )?;
        for d in [1, 2] {
            assert_eq!(pure.eval(&d)?.to_bits(), 0.0f64.to_bits());
        }

        for delta in [0.0, -0.0, 1.0] {
            let map = amplify_without_replacement(
                ChangeOneDistance,
                Approximate(MaxDivergence),
                PrivacyMap::new(move |_: &u32| (epsilon, delta)),
                10,
                10,
            )?;
            let (epsilon_out, delta_out) = map.eval(&1)?;
            assert_eq!(epsilon_out.to_bits(), 0.0f64.to_bits());
            assert_eq!(delta_out.to_bits(), delta.abs().to_bits());
            let (epsilon_out, delta_out) = map.eval(&2)?;
            assert_eq!(epsilon_out.to_bits(), 0.0f64.to_bits());
            assert_eq!(delta_out.to_bits(), delta.abs().to_bits());
        }
    }

    // Sampling the whole population, and empty populations, are distinct cases.
    let empty = amplify_without_replacement(
        ChangeOneDistance,
        MaxDivergence,
        PrivacyMap::new(|_: &u32| 0.0),
        0,
        0,
    );
    assert_eq!(
        empty.err().expect("empty population").variant,
        ErrorVariant::FailedMap
    );
    Ok(())
}

#[test]
fn test_amplify_genuine_infinity_and_error_propagation() -> Fallible<()> {
    use crate::combinators::amplify_without_replacement;
    use crate::core::PrivacyMap;
    use crate::error::ErrorVariant;
    use crate::measures::{Approximate, MaxDivergence};
    use crate::metrics::ChangeOneDistance;

    let pure = amplify_without_replacement(
        ChangeOneDistance,
        MaxDivergence,
        PrivacyMap::new(|_: &u32| f64::INFINITY),
        10,
        5,
    )?;
    assert_eq!(pure.eval(&0)?, 0.0);
    assert_eq!(pure.eval(&1)?, f64::INFINITY);
    assert_eq!(pure.eval(&2)?, f64::INFINITY);

    let approx = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (f64::INFINITY, 0.2)),
        10,
        5,
    )?;
    assert_eq!(approx.eval(&0)?, (0.0, 0.0));
    assert_eq!(approx.eval(&1)?, (f64::INFINITY, 0.1));
    assert_eq!(approx.eval(&2)?, (f64::INFINITY, 1.0));

    let approx_pure = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (f64::INFINITY, 0.0)),
        10,
        5,
    )?;
    assert_eq!(approx_pure.eval(&2)?, (f64::INFINITY, 0.0));

    // Saturation is valid only after delta was successfully computed.
    assert_eq!(super::group_privacy_approx((1.0, 0.5), 3)?.1, 1.0);

    // Non-overflow numerical failures must also propagate.
    let error = super::group_privacy_approx((f64::NAN, 0.5), 2).unwrap_err();
    assert_eq!(error.variant, ErrorVariant::NumericError);
    Ok(())
}
