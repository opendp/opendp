use super::*;
use num::Float as _;

#[test]
fn test_bool() -> Fallible<()> {
    let m_rr = make_randomized_response_bool(0.75, false)?;
    assert!(m_rr.invoke(&false).is_ok());
    assert_eq!(m_rr.map(&1)?, 3.0.ln());
    Ok(())
}

#[test]
fn test_bool_extremes() -> Fallible<()> {
    // 50% chance that the output is correct means all information is lost, query is ε=0 dp
    let m_rr = make_randomized_response_bool(0.5, false)?;
    assert_eq!(m_rr.map(&1)?, 0.0);

    // 100% chance that the output is correct is ε=inf dp
    let m_rr = make_randomized_response_bool(1.0, false)?;
    assert_eq!(m_rr.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_cat() -> Fallible<()> {
    let m_rr = make_randomized_response(HashSet::from([2, 3, 5, 6]), 0.75)?;
    assert!(m_rr.invoke(&3).is_ok());

    // (.75 * 3 / .25) = 9
    assert_eq!(m_rr.map(&1)?, 9f64.ln());
    Ok(())
}

#[test]
fn test_cat_extremes() -> Fallible<()> {
    let categories = HashSet::from([2, 3, 5, 7, 8]);
    let m_rr = make_randomized_response(categories.clone(), 1. / 5.)?;
    assert_eq!(m_rr.map(&1)?, 2.220446049250313e-16);

    let m_rr = make_randomized_response(categories, 1.)?;
    assert_eq!(m_rr.map(&1)?, f64::INFINITY);
    Ok(())
}

#[test]
fn test_bool_certified_constructor() -> Fallible<()> {
    for prob in [0.5, 0.75, 1.0] {
        let measurement = make_randomized_response_bool(prob, false)?;
        assert_eq!(measurement.map(&0)?, 0.0);
        assert!(measurement.invoke(&true).is_ok());
        assert!(measurement.invoke(&false).is_ok());
        // Arc-backed function and map remain paired after cloning.
        let cloned = measurement.clone();
        for distance in [0u32, 1u32, 2u32] {
            assert_eq!(cloned.map(&distance)?, measurement.map(&distance)?);
        }
        assert!(cloned.invoke(&true).is_ok());
    }
    assert!(make_randomized_response_bool(f64::NAN, false).is_err());
    assert!(make_randomized_response_bool(0.75, true)?.invoke(&true).is_ok());
    Ok(())
}

#[test]
fn test_bool_dyadic_53_numerator() {
    use crate::traits::samplers::dyadic53_numerator;
    let to_num = |p: f64| dyadic53_numerator(p.to_bits());
    assert_eq!(to_num(0.5), 1u64 << 52);
    assert_eq!(to_num(0.75), 3u64 << 51);
    assert_eq!(to_num(1.0), 1u64 << 53);
    assert_eq!(to_num(f64::from_bits(0.5f64.to_bits() + 1)), (1u64 << 52) + 1);
    assert_eq!(to_num(f64::from_bits(1.0f64.to_bits() - 1)), (1u64 << 53) - 1);
}

#[test]
fn test_bool_dyadic_53_extremes() -> Fallible<()> {
    use crate::traits::samplers::dyadic53_success as success;
    let max = (1u64 << 53) - 1;
    assert!(!success(0, 0));
    assert!(success(1u64 << 53, u64::MAX));
    assert!(success(1, 0));
    assert!(!success(1, 1u64 << 11));
    assert!(success(max, (max - 1) << 11));
    assert!(!success(max, max << 11));
    let deterministic = make_randomized_response_bool(1.0, false)?;
    assert_eq!(deterministic.invoke(&true)?, true);
    assert_eq!(deterministic.invoke(&false)?, false);
    Ok(())
}

#[test]
fn test_bool_rr_integer_odds_and_operator() -> Fallible<()> {
    use super::operator::BoolRR;

    for (prob, numerator) in [
        (0.5, 1u64 << 52),
        (0.75, 3u64 << 51),
        (1.0, 1u64 << 53),
        (f64::from_bits(1.0f64.to_bits() - 1), (1u64 << 53) - 1),
    ] {
        let operator = BoolRR { numerator, privacy_cost: 5.0f64 };
        assert_eq!(operator.release(true, Ok::<u64, ()>(0)), Ok(true));
        assert_eq!(operator.release(false, Ok::<u64, ()>(0)), Ok(false));
        assert_eq!(operator.release(true, Err::<u64, usize>(73)), Err(73));
        assert_eq!(operator.map(0u32, 0.0), 0.0);
        assert_eq!(operator.map(2u32, 0.0), 5.0);

        let measurement = make_randomized_response_bool(prob, false)?;
        assert_eq!(measurement.map(&0)?, 0.0);
        if prob < 1.0 {
            let factor = (numerator as f64) / ((1u64 << 53) - numerator) as f64;
            assert!(measurement.map(&1)?.exp() >= factor * (1.0 - 1e-12));
        }
    }
    Ok(())
}

#[test]
fn test_bool_rr_exact_odds_parts() {
    use crate::traits::samplers::dyadic53_odds_parts;

    assert_eq!(dyadic53_odds_parts(1u64 << 52), Some((1u64 << 52, 1u64 << 52)));
    assert_eq!(dyadic53_odds_parts(3u64 << 51), Some((3u64 << 51, 1u64 << 51)));
    assert_eq!(dyadic53_odds_parts((1u64 << 53) - 1), Some(((1u64 << 53) - 1, 1)));
    assert_eq!(dyadic53_odds_parts(1u64 << 53), None);
}

#[test]
fn test_bool_constructor_ieee_boundaries() -> Fallible<()> {
    use crate::traits::samplers::dyadic53_checked_numerator;

    let accepted = [
        0.5,
        f64::from_bits(0.5f64.to_bits() + 1),
        0.75,
        f64::from_bits(1.0f64.to_bits() - 1),
        1.0,
    ];
    for p in accepted {
        assert!(dyadic53_checked_numerator(p.to_bits()).is_some());
        let measurement = make_randomized_response_bool(p, false)?;
        assert_eq!(measurement.map(&0)?, 0.0);
        assert!(measurement.map(&1)?.is_finite() || p == 1.0);
    }

    let rejected = [
        f64::from_bits(0.5f64.to_bits() - 1),
        f64::from_bits(1.0f64.to_bits() + 1),
        0.0, -0.0, -1.0, f64::INFINITY, f64::NEG_INFINITY,
        f64::NAN, f64::from_bits(u64::MAX),
    ];
    for p in rejected {
        assert_eq!(dyadic53_checked_numerator(p.to_bits()), None);
        assert!(make_randomized_response_bool(p, false).is_err());
    }
    Ok(())
}
