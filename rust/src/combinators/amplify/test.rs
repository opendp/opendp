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
    let (_, delta_large) = amplified.eval(&u32::MAX)?;
    assert!(delta_large <= 1.0);

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

    // The group-privacy geometric series overflows, so delta must saturate.
    let amplified = amplify_without_replacement(
        ChangeOneDistance,
        Approximate(MaxDivergence),
        PrivacyMap::new(|_: &u32| (50.0, 1e-12)),
        2,
        1,
    )?;
    let (epsilon, delta) = amplified.eval(&100)?;
    assert!(epsilon.is_finite() && epsilon >= 4900.0);
    assert_eq!(delta, 1.0);

    // Exponentiation at construction may fail, or return a conservative bound.
    let large = amplify_without_replacement(
        ChangeOneDistance,
        MaxDivergence,
        PrivacyMap::new(|_: &u32| 1_000.0),
        2,
        1,
    );
    if let Ok(amplified) = large {
        if let Ok(epsilon) = amplified.eval(&1) {
            assert!(!epsilon.is_nan() && epsilon >= 999.0);
        }
    }
    Ok(())
}
